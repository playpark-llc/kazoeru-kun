# 起動方法

## コンテナ起動

ターミナルで下記コマンド実行

```
docker compose up -d
```

## ブラウザアクセス

下記URLをブラウザのアドレスバーに入力

```
localhost:3001
```

# コンテナの各サービス概要

- back
  - バックエンド (ポート: 8881)
  - APIs
    
    ### セッション関連
    1. `POST /session`
       - クリックイベント処理
       - Cookieでセッション管理
       - RedisのセッションIDに対応するカウンタをインクリメント
       - 新規セッションの場合はCookieを設定し、value=1として作成
    
    2. `PUT /session`
       - Redis→PostgreSQL同期（バックエンド内部のcronジョブから定期実行）
       - RedisのセッションデータをPostgreSQLのsession_counterテーブルにupsert
       - 既存レコードは累積カウントを更新、新規は作成
    
    3. `GET /session_counter?start_date=yyyy-mm-dd&end_date=yyyy-mm-dd`
       - セッションカウンタのサマリを取得
       - 指定期間のカウント合計を返す
       - パラメータ省略時はデフォルト期間を使用
    
    ### ステータス履歴関連
    4. `PUT /status_history`
       - ステータス履歴の更新（週次ステータス計算）
       - 先週のクリック数を集計し、目標値と比較してステータスを決定
       - status_historyテーブルに新規レコードを追加
    
    5. `GET /status_history?limit=<n>&offset=<n>&start_date=yyyy-mm-dd&end_date=yyyy-mm-dd`
       - ステータス履歴の取得（ページネーション対応）
       - 現在のステータスと履歴を返す
       - limit: 取得件数上限（デフォルト: 100）
       - offset: 取得開始位置（デフォルト: 0）
       - 日付範囲指定可能
    
    ### ヘルスチェック
    6. `GET /health`
       - システム全体のヘルスチェック
       - PostgreSQL、Redisの接続状態とレイテンシを返す
    
    7. `GET /health/liveness`
       - Kubernetesライブネスプローブ用
       - アプリケーションの生存確認
    
    8. `GET /health/readiness`
       - Kubernetes準備状態プローブ用
       - サービス提供可能状態の確認
- front
  - フロンエンド (ポート: 3001)
  - Rust/YewフレームワークによるWebAssemblyアプリケーション
  - 初回読み込み: `GET /session_counter`を呼び出して総クリック数を表示
  - クリック時: `POST /session`を呼び出してカウントを更新
- redis (ポート: 6380)
  - セッションベースのクリックカウンタキャッシュ
  - (key, value): (session_id, count) の期限付きKey-Valueストア
  - TTLはセッションCookieの有効期限と同期
- postgres (ポート: 5432)
  - 永続的なデータストレージ（PostgreSQL 16 Alpine）
  - テーブル:
    - `session_counter`: セッションごとのクリック数
    - `status_history`: 週次ステータス履歴（期間、カウント、ステータス）
  - [Database Definition](./doc/KazoeruKun/README.md)

## 定期実行タスク（バックエンド内蔵）

バックエンドアプリケーション内でtokio::spawnにより以下のタスクが自動実行されます：
- **同期ジョブ**: 60秒ごとにRedis→PostgreSQLへデータ同期
- **ステータス履歴ジョブ**: 24時間ごとに週次ステータスを更新
- 両ジョブとも自動リトライロジック付き

## シーケンス図

### ユーザ操作系

#### クリックイベント

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant Frontend as フロントエンド
    participant Backend as バックエンド
    participant Redis as Redis
    participant PostgreSQL as PostgreSQL

    User->>+Frontend: クリックイベント
    Frontend->>+Backend: POST /session
    alt CookieにセッションIDが含まれていない場合
      Backend->>Backend: セッションIDを生成
      Backend->>Backend: セッションの期限を生成
      Backend->>Backend: セッションをcookie登録
    end
    Backend->>+Redis: セッションIDインクリメント
    Redis-->>-Backend: セッションIDのカウント取得
    alt セッションIDのカウントが1(新規作成)の場合
      Backend->>+Redis: セッションIDのレコードに期限設定
      Redis->>-Backend: 期限設定確認
    end
    Backend->>-Frontend: 処理完了

```

#### 総クリック数取得

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant Frontend as フロントエンド
    participant Backend as バックエンド
    participant PostgreSQL as PostgreSQL

    User->>Frontend: ページアクセス
    Frontend->>+Backend: GET /session_counter?start_date=&end_date=
    Backend->>+PostgreSQL: session_counterテーブルから指定期間の合計取得
    PostgreSQL-->>-Backend: 総クリック数
    Backend-->>-Frontend: 総クリック数レスポンス
    Frontend->>User: 総クリック数表示
```

#### 最新ステータス取得

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant Frontend as フロントエンド
    participant Backend as バックエンド
    participant PostgreSQL as PostgreSQL

    User->>Frontend: ステータス画面アクセス
    Frontend->>+Backend: GET /status_history?limit=&offset=
    Backend->>+PostgreSQL: status_historyテーブルから最新ステータスと履歴取得
    PostgreSQL-->>-Backend: ステータスデータ
    Backend-->>-Frontend: ステータス履歴レスポンス
    Frontend->>User: 現在のステータスと履歴表示
```

### バックエンド内部の定期実行タスク

#### クリック数更新 Redis -> PostgreSQL（60秒ごと）

```mermaid
sequenceDiagram
    participant Backend as バックエンド（内部タスク）
    participant Backend as バックエンド
    participant Redis as Redis
    participant PostgreSQL as PostgreSQL

    Backend->>Backend: 内部タスクがPUT /sessionを実行
    Backend->>+Redis: 全セッションのカウント取得
    Redis-->>-Backend: カウントデータ
    Backend->>+PostgreSQL: session_counterテーブルをupsert
    PostgreSQL-->>-Backend: 更新確認
    Backend->>Backend: 次回実行をスケジュール（60秒後）
```

#### 週次ステータス更新（24時間ごと）

```mermaid
sequenceDiagram
    participant Backend as バックエンド（内部タスク）
    participant Backend as バックエンド
    participant PostgreSQL as PostgreSQL

    Backend->>Backend: 内部タスクがPUT /status_historyを実行
    Backend->>+PostgreSQL: 先週のクリック数集計
    PostgreSQL-->>-Backend: 週次カウント
    Backend->>Backend: ステータス計算（目標値と比較）
    Backend->>+PostgreSQL: status_historyテーブルに新規レコード挿入
    PostgreSQL-->>-Backend: 挿入確認
    Backend->>Backend: 次回実行をスケジュール（24時間後）
```
