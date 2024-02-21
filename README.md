# 起動方法

## コンテナ起動

ターミナルで下記コマンド実行

```
docker compose up -d
```

## ブラウザアクセス

下記URLをブラウザのアドレスバーに入力

```
localhost:3000
```

# コンテナの各サービス概要

- back
  - バックエンド
  - APIs
    1. GET /count
       - timestampsテーブルにタイムスタンプを登録し、過去1秒間のアクセス数をDBから取得
    2. POST /cleanup
       - timestampsテーブルのタイムスタンプが過去2秒よりも以前のレコードを削除
- front
  - フロンエンド
  - 1秒おきに(i)のAPIをコールして表示
- redis
  - timestampsテーブルを配置
- cron
  - 1分おきに(ii)のAPIをコール

## シーケンス図

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant Frontend as フロントエンド
    participant Backend as バックエンド
    participant Redis as Redis
    participant MySQL as MySQL

    User->>+Frontend: クリックイベント
    Frontend->>+Backend: クリックデータ送信
    Backend->>+Redis: セッションID確認
    Redis-->>-Backend: セッションIDレスポンス
    alt セッションIDが存在しない場合
        Backend->>+MySQL: セッションID新規作成
        MySQL-->>-Backend: 新規セッションID
    end
    Backend->>+Redis: クリックデータ記録
    Redis-->>-Backend: 確認
    Backend->>-Frontend: 処理完了

    loop 定期的なデータ更新
        Backend->>+Redis: クリック数カウント取得
        Redis-->>-Backend: カウントデータ
        Backend->>+MySQL: クリック数更新
        MySQL-->>-Backend: 更新確認
        Backend->>+Redis: 一時データ削除
        Redis-->>-Backend: 削除確認
    end

    loop ユーザー数更新とキャッシュ
        Backend->>+MySQL: 定期的なユーザー数取得
        MySQL-->>-Backend: ユーザー数
        Backend->>+Redis: ユーザー数キャッシュ更新
        Redis-->>-Backend: 更新確認
    end

    Frontend->>+Backend: ユーザー数リクエスト
    Backend->>+Redis: キャッシュされたユーザー数取得
    Redis-->>-Backend: ユーザー数
    Backend->>Frontend: ユーザー数レスポンス
    Frontend->>User: ユーザー数表示

    alt ユーザー数に影響する操作があった場合
        Backend->>+MySQL: ユーザー数更新
        MySQL-->>-Backend: 更新確認
        Backend->>+Redis: キャッシュ無効化/更新
        Redis-->>-Backend: 無効化/更新確認
    end
```
