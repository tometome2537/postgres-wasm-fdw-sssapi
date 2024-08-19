# Postgres Wasm FDW [Template]

This project demostrates how to create a Postgres Foreign Data Wrapper with Wasm, using the [Wrappers framework](https://github.com/supabase/wrappers).

This example reads the [realtime GitHub events](https://api.github.com/events) into a Postgres database.

## Getting started

まずは [Wasm FDW developing guide](https://fdw.dev/guides/create-wasm-wrapper/)にアクセスし
**Add wasm32-unknown-unknown target**と**Install the WebAssembly Component Model subcommand**のステップを行なってください。

Supabaseのダッシュボード>SQLエディターを開き、

```sql
create extension if not exists wrappers with schema extensions;

create foreign data wrapper wasm_wrapper
  handler wasm_fdw_handler
  validator wasm_fdw_validator;

create server sssapi_server
  foreign data wrapper wasm_wrapper
  options (
    fdw_package_url 'https://github.com/tometome2537/postgres-wasm-fdw-sssapi/releases/download/v1.0.1/wasm_fdw_sssapi.wasmwasm_fdw_example.wasm',
    fdw_package_name 'ruchi12377:sssapi-fdw',
    fdw_package_version '1.0.1',
    fdw_package_checksum ''
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

## License

[Apache License Version 2.0](./LICENSE)
