/* Make sure TS_RS_EXPORT_DIR is set! */
// re-export with:
// cargo test export_binding

pub mod lb_app_types {
    
    use ts_rs::TS;
    
    #[derive(Debug, serde::Serialize, serde::Deserialize, TS)] 
    #[ts(export, export_to="server-types.ts")]
    #[ts(rename_all = "camelCase")]
    pub struct ListInfo {
        pub list_name: String,
        pub author_user: String,
        pub attrs: Vec<String>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, export_to="server-types.ts")]
    #[ts(rename_all = "camelCase")]
    pub struct ListRow {
        pub total_rows: usize,
        pub row_data: String,
}
}


pub mod db_io_types {

    use ts_rs::TS;
    use mysql_common::chrono::{NaiveDateTime, Utc, SubsecRound};
    
    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone, TS)] 
    #[serde(rename_all = "camelCase")]
    #[ts(export, export_to="server-types.ts")]
    #[ts(rename_all = "camelCase")]
    pub struct GuestbookEntry {
        #[ts(optional)]
        #[serde(default)]
        pub id: Option<String>,
        #[ts(optional)]
        pub time_stamp: Option<NaiveDateTime>,
        pub name: String,
        pub note: String,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone, TS)] 
    #[serde(rename_all = "camelCase")]
    #[ts(export, export_to="server-types.ts")]
    #[ts(rename_all = "camelCase")]
    pub struct EntryReceipt {
        pub time_stamp: NaiveDateTime,
        pub id: String,
    }

    // This struct exists for organinzing all the JSON 
    // guestbook entries for transmission to the client into a
    // larger JSON object
    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, TS)]
    #[ts(export, export_to="server-types.ts")]
    pub struct Guestbook {
        pub guestbook: Vec<GuestbookEntry>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone, TS)]
    #[serde(rename_all = "snake_case")]
    #[ts(export, export_to="server-types.ts")]
    #[ts(rename_all = "camelCase")]
    pub struct WebpageHit {
        pub time_stamp: NaiveDateTime,
        pub user_agent: String,
    }

    impl Default for WebpageHit {
        fn default() -> WebpageHit {
            WebpageHit { 
                time_stamp: Utc::now()
                    .naive_utc()
                    .trunc_subsecs(0), 
                user_agent: String::from("Mozilla user agent") 
            }
        }
    }
}