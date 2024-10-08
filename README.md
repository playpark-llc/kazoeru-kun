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
    1. POST /session -b 'session=<SESSION_ID>'
       - click event
       - redisのkey:<SESSION_ID>に対応するvalueをカウントアップ。keyが存在しなければvalue=1としてレコード作成
    2. PUT /session
       - 定期実行
       - redisのkey, valueからMySQLのテーブルSessionCounterをupsert
       - redisのvalueがSessionCounterのsessionIdに存在する場合、対象レコードを(key, value + count) -> (sessionId, count)として更新
       - redisのvalueがSessionCounterのsessionIdに存在しなければ、(key, value) -> (sessionId, count)として新規レコード作成
    3. POST /count
       - 定期実行
       - SessionCounter.countの合計をCountSummaryに1レコード追加
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
    Backend->>+Redis: セッションIDインクリメント
    Redis-->>-Backend: セッションIDのカウント取得
    alt カウント= 1(セッションIDが存在しなかった場合)
        Backend->>+MySQL: セッションIDに対応するレコードを取得
        MySQL-->>-Backend: レコード
      alt レコードが存在する場合
        Backend->>+Redis: レコードから取得したカウントでインクリメント
        Redis-->>-Backend: セッションIDのカウント取得
      else レコードが存在しない場合
        Backend->>+MySQL: セッションID新規作成
      end
    end
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

## Table Definitions

[session_counter](./doc/KazoeruKun/session_counter.md)

[count_summary](./doc/KazoeruKun/count_summary.md)
