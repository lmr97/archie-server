use std::{fs::File, io::Read};
use mysql_common::serde_json;
use tokio;
use reqwest::{Certificate, header, StatusCode};
use custom_backend::{
    utils::init_utils::{
        get_env_var,
        process_cli_args, 
        RunMode
    },
    srv_io::db_io::{
        Guestbook,
        GuestbookEntry
    }
};

#[tokio::main]
async fn main() {

    /* Sort out protocol to use */
    let protocol = match process_cli_args().unwrap() {
        RunMode::NoTls => "http",
        _ => "https"
    };
    let domain = get_env_var("CLIENT_SOCKET").unwrap();
    let url    = format!("{protocol}://{domain}/guestbook/entries");

    /* Configure client */
    let mut cont_type_header = header::HeaderMap::new();
    cont_type_header.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str("application/json").unwrap()
    );
    let client_base = reqwest::Client::builder()
        .default_headers(cont_type_header);

    let client = if protocol == "https" {
        
        let client_pk_file = get_env_var("CLIENT_PK_FILE").unwrap();
        let mut pk_buf = Vec::new();
        File::open(client_pk_file)
            .unwrap()
            .read_to_end(&mut pk_buf)
            .unwrap();
        
        let cert = Certificate::from_pem(&pk_buf).unwrap();
        client_base.add_root_certificate(cert).build().unwrap()
    
    } else {
        client_base.build().unwrap()
    };

    post_valid_entry(&client, &url).await;
    getting_guestbook(&client, &url).await;
    post_overlong_entry_note(&client, &url).await;
    post_overlong_entry_name(&client, &url).await;
}

async fn post_valid_entry(client: &reqwest::Client, url: &String) {
    
    let valid_entry = GuestbookEntry {
        name: String::from("a normal name"),
        note: String::from(
            "A moderately sized note. Not much special going on here, \
            aside from some non-ASCII Unicode (UTF-8): ગુજરાતી લિપિ \
            (this is Gujarati!)."
        )
    };

    let resp = client
        .post(url)
        .body(serde_json::to_string(&valid_entry).unwrap())
        .send()
        .await
        .unwrap();

    // only checking whether the post was successful; 
    // whether it's retrievable is tested in getting_guestbook()
    assert_eq!(resp.status(), StatusCode::OK);
}

async fn getting_guestbook(client: &reqwest::Client, url: &String) {

    // since I don't know when the extra entries were added,
    // and the accuracy is checked in the get_guestbook() unit test
    // we're only comparing content without timestamps

    let latest_entry = GuestbookEntry {
        name: String::from("a normal name"),
        note: String::from(
            "A moderately sized note. Not much special going on here, \
            aside from some non-ASCII Unicode (UTF-8): ગુજરાતી લિપિ \
            (this is Gujarati!)."
        )
    };
    let test_guestbook_vec0 = vec![
        latest_entry.clone(),
        GuestbookEntry {
            name: String::from("(anonymous)"),
            note: String::new()
        },
        GuestbookEntry {
            name: String::from("Lettuce % % \\% \\' break some sTuff ⌠ 	⌡ 	⌢ 	⌣ 	⌤"),
            note: String::from(
                "ᏣᎳᎩ ᎦᏬᏂᎯᏍᏗ (Cherokee!) \n\\\\% %%' ''\\n\
                മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി \
                ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് \
                വിവേകബുദ്ധിയും മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌ \
                (this says 'All human beings are born free and equal in dignity and rights. \
                They are endowed with reason and conscience and should act towards one \
                another in a spirit of brotherhood.' in Malayalam. It comes from the \
                UN's Universal Declaration on Human Rights)"
            )
        },
        GuestbookEntry {
            name: String::from("约翰·塞纳"),
            note: String::from("我很喜欢冰淇淋")
        },
        GuestbookEntry {
            name: String::from("Linus"),
            note: String::from("nice os choice!")
        },
        GuestbookEntry {
            name: String::from("(anonymous)"),
            note: String::from("you'll never know...")
        },
        GuestbookEntry {
            name: String::from("Ada"),
            note: String::from("It's so nice to be here!")
        },
    ];

    // first entry may be duplicated, depending on whether this is run
    // the first time (no TLS) or the second time (with TLS), since 
    // post_valid_entry (only) runs before this function
    let mut test_guestbook_vec1 = test_guestbook_vec0.clone(); 
    test_guestbook_vec1.insert(0, latest_entry);

    let resp = client
        .get(url)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // remove timestamps; we can't know all their values
    // when this function is run
    let resp_body = resp.text()
        .await
        .unwrap();
    let gotten_guestbook: Guestbook = serde_json::from_str(&resp_body).unwrap();
    let gb_no_ts_vec: Vec<GuestbookEntry> = gotten_guestbook.guestbook
        .into_iter()
        .map(|ent| {
        GuestbookEntry {
            name: ent.name,
            note: ent.note
        }
    }).collect();

    assert!(test_guestbook_vec0 == gb_no_ts_vec || test_guestbook_vec1 == gb_no_ts_vec);
}

async fn post_overlong_entry_note(client: &reqwest::Client, url: &String) {

    // gonna get real weird with it
    let overlong_entry = GuestbookEntry {
        name: String::from("A resonable name"),
        note: String::from(
            "ᏣᎳᎩ ᎦᏬᏂᎯᏍᏗ (this is Cherokee!) \n\\\\% %%' ''

            മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി 
            ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് 
            വിവേകബുദ്ധിയും മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌
            (this says 'All human beings are born free and equal in dignity and rights. 
            They are endowed with reason and conscience and should act towards one 
            another in a spirit of brotherhood.' in Malayalam. It comes from the 
            UN's Universal Declaration on Human Rights)
            
            Let's stick with this and go further. We need to make sure we have this
            data exceed 1KB. And now it does."
        ),
    };

    let resp = client
        .post(url)
        .body(serde_json::to_string(&overlong_entry).unwrap())
        .send()
        .await
        .unwrap();

    // should reject with a 413
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
    

async fn post_overlong_entry_name(client: &reqwest::Client, url: &String) {

    // gonna get real weird with it
    let overlong_name = GuestbookEntry {
        name: String::from(
            "A name മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും that is too ᎦᏬᏂᎯᏍᏗ long.
            so long, in fact, I needed to add all this stuff!"),
        note: String::from("a brief note"),
    };

    let resp = client
        .post(url)
        .body(serde_json::to_string(&overlong_name).unwrap())
        .send()
        .await
        .unwrap();

    // should reject with a 413
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}