# Supabase FDW For SSSApi

このリポジトリはSSSApiをSupabaseの外部データベースとして扱うためのFDWを提供します。

## SSSApiとは

[SSSApi]("https://sssapi.app/")とはGoogle Spread SheetをマスタデータとするJSONを配信するAPIサーバーを提供してるサービスです。
Google Spread Sheetをマスタデータとしてるため、非エンジニアでもデータの変更がしやすく、ファイルの権限もGoogleアカウントを用いて簡単に共有できるメリットがあります。

## 使い方

まずは [Wasm FDW developing guide](https://fdw.dev/guides/create-wasm-wrapper/)にアクセスし
**Add wasm32-unknown-unknown target**と**Install the WebAssembly Component Model subcommand**の２ステップを行なってください。

次にSupabaseのダッシュボード>SQLエディターを開き、

```sql
create extension if not exists wrappers with schema extensions;

create foreign data wrapper wasm_wrapper
  handler wasm_fdw_handler
  validator wasm_fdw_validator;

create server sssapi_server
  foreign data wrapper wasm_wrapper
  options (
    fdw_package_url 'https://github.com/tometome2537/postgres-wasm-fdw-sssapi/releases/download/v1.0.2/wasm_fdw_sssapi.wasmwasm_fdw_example.wasm',
    fdw_package_name 'ruchi12377:sssapi-fdw',
    fdw_package_version '1.0.2',
    fdw_package_checksum '80e443224345f08283a653b881b60888eac09332bd655b76f9cb544518ff514d'
    );

create schema if not exists spreadsheet;

create foreign table spreadsheet.entity (
  id TEXT,
  public BOOLEAN,
  -- カラム名がキャメルケースの場合は""で囲む。
  -- カラム名が勝手に小文字に変更されてしまうので。
  "fullName" TEXT,
)
server sssapi_server
options (
  -- ここにAPIのIDを入れる
  sssapi_id ''
);

```

を実行してください。その際、テーブル名やテーブルのカラムは使用したいスプレッドシートに合わせて適宜変更してください。

## 連絡先

開発者のTwitter: [Ruchi12377](https://twitter.com/ruchi12377) (English Ok!)

## License

[Apache License Version 2.0](./LICENSE)
