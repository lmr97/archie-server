use mysql_common::serde_json;
use tokio;
use reqwest::StatusCode;
use custom_backend::types::db_io_types::{EntryReceipt, Guestbook, GuestbookEntry};
mod client_config;

// WARNING: these tests will fail if they are run before the library tests.
// These tests expect the data posted to the data during those tests
// to be there.

#[tokio::main]
async fn main() {

    let (protocol, mut url) = client_config::get_base_url();
    url.push_str("/guestbook/entries");
    let client = client_config::config_client(protocol);

    post_valid_entry(&client, &url).await;
    getting_guestbook(&client, &url).await;
    post_overlong_entry_note(&client, &url).await;
    post_overlong_entry_name(&client, &url).await;
}


// This checks whether the entry can be deserialized from camelCase,
// and that an EntryReceipt is sent
async fn post_valid_entry(client: &reqwest::Client, url: &String) {
    
    let valid_entry = String::from(
        "{\"id\":null,\"timeStamp\":null,\"name\": \"a normal name\",\"note\": \"Some non-ASCII Unicode: ગુજરાતી લિપિ.\"}"
    );

    let resp = client
        .post(url)
        .body(valid_entry)
        .send()
        .await
        .unwrap();

    // only checking whether the post was successful, and an EntryReceipt
    // was received. whether it's retrievable is tested in getting_guestbook()
    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body_text = resp.text().await.unwrap();

    // test both that the response body can be deserialized into an EntryReceipt
    // and that it is camelCase
    assert!(resp_body_text.contains("timeStamp"));
    serde_json::from_str::<EntryReceipt>(&resp_body_text).unwrap();  
}

async fn getting_guestbook(client: &reqwest::Client, url: &String) {

    // since I don't know when the extra entries were added,
    // and the accuracy is checked in the get_guestbook() unit test
    // we're only comparing content without timestamps

    let latest_entry = GuestbookEntry {
        id: None,
		time_stamp: None,
        name: String::from("a normal name"),
        note: String::from("Some non-ASCII Unicode: ગુજરાતી લિપિ.")
    };
    let test_guestbook_vec0 = vec![
        latest_entry.clone(),
        GuestbookEntry {
            id: None,
            time_stamp: None,
            name: String::from("(anonymous)"),
            note: String::new()
        },
        GuestbookEntry {
            id: None,
            time_stamp: None,
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
            id: None,
            time_stamp: None,
            name: String::from("约翰·塞纳"),
            note: String::from("我很喜欢冰淇淋")
        },
        GuestbookEntry {
            id: None,
            time_stamp: None,
            name: String::from("Linus"),
            note: String::from("nice os choice!")
        },
        GuestbookEntry {
            id: None,
            time_stamp: None,
            name: String::from("(anonymous)"),
            note: String::from("you'll never know...")
        },
        GuestbookEntry {
            id: None,
            time_stamp: None,
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

    // remove timestamps and IDs; we can't know all their values
    // when this function is run
    let resp_body = resp.text()
        .await
        .unwrap();

    // check for camelCase serialization
    assert!(resp_body.contains("timeStamp"));
    
    let gotten_guestbook: Guestbook = serde_json::from_str(&resp_body).unwrap();
    let gb_no_ts_vec: Vec<GuestbookEntry> = gotten_guestbook.guestbook
        .into_iter()
        .map(|ent| {

            GuestbookEntry {
                id: None,
                time_stamp: None,
                name: ent.name,
                note: ent.note
            }
        })
        .collect();

    assert!(test_guestbook_vec0 == gb_no_ts_vec || test_guestbook_vec1 == gb_no_ts_vec);
}

async fn post_overlong_entry_note(client: &reqwest::Client, url: &String) {

    // gonna get real weird with it
    let overlong_entry = GuestbookEntry {
        id: None,
		time_stamp: None,
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
        id: None,
		time_stamp: None,
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
