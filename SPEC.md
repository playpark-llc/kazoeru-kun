# SPEC.md - Issue #70: README.md修正

## 概要
backendのAPI周りの修正や追加機能をREADME.mdに反映する。

## 受け入れ基準 (Acceptance Criteria)

- [ ] README.mdの「API Endpoints」セクションが現在実装されているすべてのエンドポイントを正確に記載している
- [ ] 各APIエンドポイントについて、HTTPメソッド、パス、パラメータ、レスポンス形式が明記されている
- [ ] 最近追加されたステータス履歴関連のエンドポイント（PUT /status_history、GET /status_history）が文書化されている
- [ ] APIエンドポイントの説明が実際の実装と一致している
- [ ] データフローの説明が現在のcronジョブの動作（週次ステータス更新を含む）を反映している
- [ ] README.mdのDatabase Schemaセクションにstatus_historyテーブルが追加されている

## 対象外 (Non-Goals)

- フロントエンドのコード変更
- バックエンドAPIの実装変更
- 新機能の追加
- README.md以外のドキュメントの修正
- テストコードの追加（README.mdの正確性は手動レビューで確認）

## 実装方針

1. backend/src/main.rsおよび関連ファイルを確認し、現在のAPIエンドポイントをすべて洗い出す
2. README.mdの現在の記載内容と比較し、不足・誤りを特定する
3. README.mdを更新し、すべてのAPIエンドポイントと最新の仕様を反映する