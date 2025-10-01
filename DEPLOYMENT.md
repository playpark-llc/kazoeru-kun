# Kazoeru-kun デプロイガイド

このドキュメントは、Supabase (PostgreSQL) と Upstash (Redis) を利用しながら、Kazoeru-kun を Fly.io 上に本番デプロイするための手順をまとめたものです。

## アーキテクチャ

```
┌─────────────────┐
│   Fly.io App    │
│  (kazoeru-kun)  │
│                 │
│  Backend(Rust)  │
│  Frontend(Yew)  │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌─────────┐
│Supabase│ │ Upstash │
│  (DB)  │ │ (Redis) │
└────────┘ └─────────┘
```

## 前提条件

1. **Supabase アカウント**
   - https://supabase.com でサインアップ
   - 新しいプロジェクトを作成（リージョンは Tokyo 推奨）

2. **Upstash アカウント**
   - https://console.upstash.com でサインアップ
   - Tokyo (ap-northeast-1) リージョンで Redis データベースを作成

3. **Fly.io アカウント**
   - https://fly.io でサインアップ
   - Fly CLI をインストール: `curl -L https://fly.io/install.sh | sh`
   - `fly auth login` で認証

## Step 1: Supabase の準備

1. Supabase プロジェクトを作成
2. Settings → Database から接続情報 (Connection string) を取得
   - Fly からはコネクションプーラー (port 6543 または 6432) を利用
3. マイグレーションを実行:
   ```bash
   cd app/backend/migration
   DATABASE_URL="postgresql://postgres:[PASSWORD]@db.[PROJECT_ID].supabase.co:5432/postgres"    sea-orm-cli migrate up
   ```
   ※ Supabase の接続文字列が IPv6 のみを返す場合、必要に応じて VPN または別環境から実行してください。

## Step 2: Upstash Redis の準備

1. Upstash にログインし、新しい Redis データベースを作成
2. リージョンは Tokyo (ap-northeast-1) を選択
3. ダッシュボードから Redis URL をコピー
4. 接続確認 (任意):
   ```bash
   redis-cli -u "redis://default:xxxx@xxxxx.upstash.io:6379" ping
   ```

## Step 3: 環境変数の設定

1. テンプレートをコピー:
   ```bash
   cp app/backend/.env.production.example app/backend/.env.production
   ```

2. `app/backend/.env.production` を編集して実値を設定:
   ```env
   DATABASE_URL=postgresql://postgres:[PASSWORD]@aws-1-ap-northeast-1.pooler.supabase.com:6543/postgres
   REDIS_URL=redis://default:[PASSWORD]@[ENDPOINT].upstash.io:6379
   ALLOWED_ORIGINS=https://kazoeru-kun.fly.dev
   TARGET_VALUE=100000
   SYNC_JOB_INTERVAL_SECS=60
   STATUS_HISTORY_JOB_INTERVAL_SECS=86400
   ROCKET_SECRET_KEY="<32バイトのランダム文字列>"
   ```
   - `ROCKET_SECRET_KEY` は `openssl rand -base64 32` などで生成します。
   - Fly では `.env.production` を直接参照しないため、最終的には `fly secrets set` に登録します。

## Step 4: Fly.io へのデプロイ

1. **アプリ初期化 (未実行の場合のみ)**
   ```bash
   cd app/backend
   fly launch --name kazoeru-kun --region nrt --no-deploy
   ```

2. **シークレットの登録**
   ```bash
   fly secrets set \
     DATABASE_URL="postgresql://..." \
     REDIS_URL="redis://default:..." \
     ALLOWED_ORIGINS="https://kazoeru-kun.fly.dev" \
     TARGET_VALUE="100000" \
     SYNC_JOB_INTERVAL_SECS="60" \
     STATUS_HISTORY_JOB_INTERVAL_SECS="86400" \
     ROCKET_SECRET_KEY="<生成したキー>"
   ```

   **シークレット検証**
   ```bash
   fly secrets list | grep ROCKET_SECRET_KEY
   ```

3. **デプロイ**
   ```bash
   fly deploy
   ```

4. **状態確認**
   ```bash
   fly status
   fly logs
   ```

5. **アプリにアクセス**
   - `https://kazoeru-kun.fly.dev` をブラウザで確認
   - `fly open` でもアクセス可能

## テスト戦略

### 開発用テスト
ローカル PostgreSQL と Redis を含む標準の `docker-compose.yml` を使用:
```bash
docker compose up
```

### 本番ビルドテスト

#### ⚠️ macOS の Docker 制限
Docker Desktop (macOS) は標準で IPv6 をサポートしていないため、Supabase が IPv6 のみのアドレスを返すケースでは `NetworkUnreachable` エラーになる可能性があります。

#### 回避策例
- IPv6 が利用できるマシンや CI でテストを実行
- Fly.io の機能を用いた Staging デプロイで検証

#### 本番設定でのローカル起動 (外部接続が可能な環境向け)
1. `.env.production` に実際の接続情報を設定
2. イメージをビルド:
   ```bash
   docker compose -f docker-compose.prod.yml build
   ```
3. 起動:
   ```bash
   docker compose -f docker-compose.prod.yml up
   # ブラウザで http://localhost:3001
   ```

## 監視

### Fly.io
```bash
fly logs            # ログ確認
fly status          # 稼働状況
fly dashboard       # メトリクス (ブラウザ)
```

### Supabase
- https://supabase.com/dashboard/project/[PROJECT_ID]
- 接続数やクエリ状況をモニタリング

### Upstash
- https://console.upstash.com
- コマンド使用量やレイテンシを確認 (無料枠は月 50 万コマンド)

## トラブルシューティング

- **DB 接続エラー**: プーリング接続を使用しているか、マイグレーションが完了しているか、Fly のシークレットが正しいか確認
- **Redis 接続エラー**: Upstash のサービスステータスと URL を確認し、月間コマンド数が上限に達していないかチェック
- **デプロイ失敗**: `fly logs -n 100` で詳細を確認し、必要に応じて `fly ssh console` で内部に入り調査。`fly secrets list` でシークレットセット状況も確認

## コスト管理

| サービス | 無料枠 | 備考 |
|----------|--------|------|
| Supabase | 500MB ストレージ / 50K MAU | 小規模なら十分 |
| Upstash  | 256MB / 50 万コマンド | コマンド上限に注意 |
| Fly.io   | 小型インスタンスで月 ~5USD | トラフィックに応じて増加 |

**概算月額**: 0〜5 USD 程度 (無料枠内での運用を想定)

## バックアップと復旧

- **Supabase**: 無料プランでも日次バックアップが自動取得。ダッシュボードから復元可能
- **Upstash**: 必要に応じて REST API でエクスポートし、定期的にキャッシュのプリウォームを検討

## セキュリティチェックリスト

- [ ] 環境変数はすべて `fly secrets` 経由で設定 (リポジトリに含めない)
- [ ] 本番ドメインに合わせて CORS を設定
- [ ] データベースはプーリング接続を使用
- [ ] Fly.io 側で HTTPS を強制
- [ ] シークレット値を再利用せず、権限を最小化

## アプリケーション更新手順

1. ローカルで変更を加える
2. 必要に応じて本番設定でテスト
3. デプロイ:
   ```bash
   fly deploy
   ```
4. デプロイ監視:
   ```bash
   fly logs --tail
   ```

## ロールバック

デプロイに問題が起きた場合:
```bash
fly releases      # 過去リリースを一覧
fly rollback <VERSION>  # 指定したバージョンへロールバック
```
