#[allow(warnings)]
mod bindings;

use serde_json::Value as JsonValue;

use bindings::{
    exports::supabase::wrappers::routines::Guest,
    supabase::wrappers::{
        http,
        types::{Cell, Context, FdwError, FdwResult, OptionsType, Row, TypeOid},
        utils,
    },
};

#[derive(Debug, Default)]
struct SSSApiFdw {
    base_url: String,
    src_rows: Vec<JsonValue>,
    src_idx: usize,
}

// pointer for the static FDW instance
static mut INSTANCE: *mut SSSApiFdw = std::ptr::null_mut::<SSSApiFdw>();

impl SSSApiFdw {
    // initialise FDW instance
    fn init_instance() {
        let instance = Self::default();
        unsafe {
            INSTANCE = Box::leak(Box::new(instance));
        }
    }

    fn this_mut() -> &'static mut Self {
        unsafe { &mut (*INSTANCE) }
    }
}

impl Guest for SSSApiFdw {
    fn host_version_requirement() -> String {
        // semver expression for Wasm FDW host version requirement
        // ref: https://docs.rs/semver/latest/semver/enum.Op.html
        "^1.0.0".to_string()
    }

    fn init(ctx: &Context) -> FdwResult {
        Self::init_instance();
        let this = Self::this_mut();

        // get API URL from foreign server options if it is specified
        let opts = ctx.get_options(OptionsType::Server);
        this.base_url = opts.require_or("base_url", "https://api.sssapi.app");

        Ok(())
    }

    fn begin_scan(ctx: &Context) -> FdwResult {
        let this = Self::this_mut();

        // get sheet id from foreign table options and make the request URL
        let opts = ctx.get_options(OptionsType::Table);
        let sssapi_id = opts.require("sssapi_id")?;
        let url = format!("{}/{}", this.base_url, sssapi_id);

        // make up request headers
        let headers: Vec<(String, String)> =
            vec![("user-agent".to_owned(), "Sheets FDW".to_owned())];

        // make a request to Google API and parse response as JSON
        let req = http::Request {
            method: http::Method::Get,
            url,
            headers,
            body: String::default(),
        };
        let resp = http::get(&req)?;
        // remove invalid prefix from response to make a valid JSON string
        let body = resp.body.as_str();
        let resp_json: JsonValue = serde_json::from_str(body).map_err(|e| e.to_string())?;

        // extract source rows from response
        this.src_rows = resp_json
            .as_array()
            .map(|v| v.to_owned())
            .expect("response should be a JSON array");

        // output a Postgres INFO to user (visible in psql), also useful for debugging
        utils::report_info(&format!(
            "We got response array length: {}",
            this.src_rows.len()
        ));

        Ok(())
    }

    fn iter_scan(ctx: &Context, row: &Row) -> Result<Option<u32>, FdwError> {
        let this = Self::this_mut();

        // if all source rows are consumed, stop data scan
        if this.src_idx >= this.src_rows.len() {
            return Ok(None);
        }

        let src_row = &this.src_rows[this.src_idx];

        // loop through each target column, map source cell to target cell
        for tgt_col in ctx.get_columns() {
            let (tgt_col_num, tgt_col_name) = (tgt_col.num(), tgt_col.name());
            if let Some(src) = src_row.get(tgt_col_name.to_owned()) {
                // we only support I64 and String cell types here, add more type
                // conversions if you need
                let cell = match tgt_col.type_oid() {
                    TypeOid::Bool => src.as_bool().map(|v| Cell::Bool(v)),
                    TypeOid::I64 => src.as_f64().map(|v| Cell::I64(v as _)),
                    TypeOid::String => src.as_str().map(|v| Cell::String(v.to_owned())),
                    _ => {
                        return Err(format!(
                            "column {} data type is not supported",
                            tgt_col_name
                        ));
                    }
                };

                //TODO: Add some types
                // let v = match tgt_col.type_oid() {
                //     TypeOid::F64 => "F64",
                //     TypeOid::Numeric => "Numeric",
                //     TypeOid::Date => "Date",
                //     TypeOid::Timestamp => "Timestamp",
                //     TypeOid::Timestamptz => "Timestamptz",
                //     TypeOid::Json => "Json",
                // };

                // push the cell to target row
                row.push(cell.as_ref());
            } else {
                row.push(None);
            }
        }

        // advance to next source row
        this.src_idx += 1;

        // tell Postgres we've done one row, and need to scan the next row
        Ok(Some(0))
    }

    fn re_scan(_ctx: &Context) -> FdwResult {
        Err("re_scan on foreign table is not supported".to_owned())
    }

    fn end_scan(_ctx: &Context) -> FdwResult {
        let this = Self::this_mut();
        this.src_rows.clear();
        Ok(())
    }

    fn begin_modify(_ctx: &Context) -> FdwResult {
        Err("modify on foreign table is not supported".to_owned())
    }

    fn insert(_ctx: &Context, _row: &Row) -> FdwResult {
        Ok(())
    }

    fn update(_ctx: &Context, _rowid: Cell, _row: &Row) -> FdwResult {
        Ok(())
    }

    fn delete(_ctx: &Context, _rowid: Cell) -> FdwResult {
        Ok(())
    }

    fn end_modify(_ctx: &Context) -> FdwResult {
        Ok(())
    }
}

bindings::export!(SSSApiFdw with_types_in bindings);
