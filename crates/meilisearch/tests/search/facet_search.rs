use meili_snap::snapshot;
use once_cell::sync::Lazy;

use crate::common::{Server, Value};
use crate::json;

static DOCUMENTS: Lazy<Value> = Lazy::new(|| {
    json!([
        {
            "title": "Shazam!",
            "genres": ["Action", "Adventure"],
            "id": "287947",
        },
        {
            "title": "Captain Marvel",
            "genres": ["Action", "Adventure"],
            "id": "299537",
        },
        {
            "title": "Escape Room",
            "genres": ["Horror", "Thriller", "Multiple Words"],
            "id": "522681",
        },
        {
            "title": "How to Train Your Dragon: The Hidden World",
            "genres": ["Action", "Comedy"],
            "id": "166428",
        },
        {
            "title": "Gläss",
            "genres": ["Thriller"],
            "id": "450465",
        }
    ])
});

#[actix_rt::test]
async fn simple_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(dbg!(response)["facetHits"].as_array().unwrap().len(), 2);

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "adventure"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(response["facetHits"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn advanced_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    index.update_settings_typo_tolerance(json!({ "enabled": false })).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "adventre"})).await;

    snapshot!(code, @"200 OK");
    snapshot!(response["facetHits"].as_array().unwrap().len(), @"0");

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "àdventure"})).await;

    snapshot!(code, @"200 OK");
    snapshot!(response["facetHits"].as_array().unwrap().len(), @"1");
}

#[actix_rt::test]
async fn more_advanced_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    index.update_settings_typo_tolerance(json!({ "disableOnWords": ["adventre"] })).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "adventre"})).await;

    snapshot!(code, @"200 OK");
    snapshot!(response["facetHits"].as_array().unwrap().len(), @"0");

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "adventure"})).await;

    snapshot!(code, @"200 OK");
    snapshot!(response["facetHits"].as_array().unwrap().len(), @"1");
}

#[actix_rt::test]
async fn simple_facet_search_with_max_values() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_faceting(json!({ "maxValuesPerFacet": 1 })).await;
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(dbg!(response)["facetHits"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn simple_facet_search_by_count_with_max_values() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index
        .update_settings_faceting(
            json!({ "maxValuesPerFacet": 1, "sortFacetValuesBy": { "*": "count" } }),
        )
        .await;
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(dbg!(response)["facetHits"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn non_filterable_facet_search_error() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;
    assert_eq!(code, 400, "{}", response);

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "adv"})).await;
    assert_eq!(code, 400, "{}", response);
}

#[actix_rt::test]
async fn facet_search_dont_support_words() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "words"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(response["facetHits"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn simple_facet_search_with_sort_by_count() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.update_settings_faceting(json!({ "sortFacetValuesBy": { "*": "count" } })).await;
    index.update_settings_filterable_attributes(json!(["genres"])).await;
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    let hits = response["facetHits"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0], json!({ "value": "Action", "count": 3 }));
    assert_eq!(hits[1], json!({ "value": "Adventure", "count": 2 }));
}

#[actix_rt::test]
async fn add_documents_and_deactivate_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;
    let (response, code) = index
        .update_settings(json!({
            "facetSearch": false,
            "filterableAttributes": ["genres"],
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 400, "{}", response);
    snapshot!(response, @r###"
    {
      "message": "Facet search is disabled for this index",
      "code": "invalid_search_disabled_facet_search",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_disabled_facet_search"
    }
    "###);
}

#[actix_rt::test]
async fn deactivate_facet_search_and_add_documents() {
    let server = Server::new().await;
    let index = server.index("test");

    let (response, code) = index
        .update_settings(json!({
            "facetSearch": false,
            "filterableAttributes": ["genres"],
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;
    let documents = DOCUMENTS.clone();
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 400, "{}", response);
    snapshot!(response, @r###"
    {
      "message": "Facet search is disabled for this index",
      "code": "invalid_search_disabled_facet_search",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_disabled_facet_search"
    }
    "###);
}

#[actix_rt::test]
async fn deactivate_facet_search_add_documents_and_activate_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let (response, code) = index
        .update_settings(json!({
            "facetSearch": false,
            "filterableAttributes": ["genres"],
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;
    let documents = DOCUMENTS.clone();
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) = index
        .update_settings(json!({
            "facetSearch": true,
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(dbg!(response)["facetHits"].as_array().unwrap().len(), 2);
}

#[actix_rt::test]
async fn deactivate_facet_search_add_documents_and_reset_facet_search() {
    let server = Server::new().await;
    let index = server.index("test");

    let (response, code) = index
        .update_settings(json!({
            "facetSearch": false,
            "filterableAttributes": ["genres"],
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;
    let documents = DOCUMENTS.clone();
    let (response, _code) = index.add_documents(documents, None).await;
    index.wait_task(response.uid()).await;

    let (response, code) = index
        .update_settings(json!({
            "facetSearch": serde_json::Value::Null,
        }))
        .await;
    assert_eq!("202", code.as_str(), "{:?}", response);
    index.wait_task(response.uid()).await;

    let (response, code) =
        index.facet_search(json!({"facetName": "genres", "facetQuery": "a"})).await;

    assert_eq!(code, 200, "{}", response);
    assert_eq!(dbg!(response)["facetHits"].as_array().unwrap().len(), 2);
}
