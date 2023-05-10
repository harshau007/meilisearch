use meili_snap::{json_string, snapshot};
use serde_json::json;

use crate::common::{GetAllDocumentsOptions, Server};

#[actix_rt::test]
async fn delete_one_document_unexisting_index() {
    let server = Server::new().await;
    let index = server.index("test");
    let task = index.delete_document(0).await;
    let task = task.wait_for_completion().await;
    snapshot!(task, @"failed");
}

#[actix_rt::test]
async fn delete_one_unexisting_document() {
    let server = Server::new().await;
    let index = server.index("test");
    index.create(None).await;
    let task = index.delete_document(0).await;
    let task = task.wait_for_completion().await;
    snapshot!(task, @"succeeded");
}

#[actix_rt::test]
async fn delete_one_document() {
    let server = Server::new().await;
    let index = server.index("test");
    let task = index.add_documents(json!([{ "id": 0, "content": "foobar" }]), None).await;
    task.wait_for_completion();
    let task = server.index("test").delete_document(0).await;
    task.wait_for_completion();

    let (_response, code) = index.get_document(0, None).await;
    snapshot!(code, @"404");
}

#[actix_rt::test]
async fn clear_all_documents_unexisting_index() {
    let server = Server::new().await;
    let index = server.index("test");
    let (_response, code) = index.clear_all_documents().await;
    assert_eq!(code, 202);

    let response = index.wait_task(0).await;

    assert_eq!(response["status"], "failed");
}

#[actix_rt::test]
async fn clear_all_documents() {
    let server = Server::new().await;
    let index = server.index("test");
    index
        .add_documents(
            json!([{ "id": 1, "content": "foobar" }, { "id": 0, "content": "foobar" }]),
            None,
        )
        .await;
    index.wait_task(0).await;
    let (_response, code) = index.clear_all_documents().await;
    assert_eq!(code, 202);

    let _update = index.wait_task(1).await;
    let (response, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    assert_eq!(code, 200);
    assert!(response["results"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn clear_all_documents_empty_index() {
    let server = Server::new().await;
    let index = server.index("test");
    index.create(None).await;

    let (_response, code) = index.clear_all_documents().await;
    assert_eq!(code, 202);

    let _update = index.wait_task(0).await;
    let (response, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    assert_eq!(code, 200);
    assert!(response["results"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn error_delete_batch_unexisting_index() {
    let server = Server::new().await;
    let index = server.index("test");
    let (_, code) = index.delete_batch(vec![]).await;
    let expected_response = json!({
        "message": "Index `test` not found.",
        "code": "index_not_found",
        "type": "invalid_request",
        "link": "https://docs.meilisearch.com/errors#index_not_found"
    });
    assert_eq!(code, 202);

    let response = index.wait_task(0).await;

    assert_eq!(response["status"], "failed");
    assert_eq!(response["error"], expected_response);
}

#[actix_rt::test]
async fn delete_batch() {
    let server = Server::new().await;
    let index = server.index("test");
    index.add_documents(json!([{ "id": 1, "content": "foobar" }, { "id": 0, "content": "foobar" }, { "id": 3, "content": "foobar" }]), Some("id")).await;
    index.wait_task(0).await;
    let (_response, code) = index.delete_batch(vec![1, 0]).await;
    assert_eq!(code, 202);

    let _update = index.wait_task(1).await;
    let (response, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    assert_eq!(code, 200);
    assert_eq!(response["results"].as_array().unwrap().len(), 1);
    assert_eq!(response["results"][0]["id"], json!(3));
}

#[actix_rt::test]
async fn delete_no_document_batch() {
    let server = Server::new().await;
    let index = server.index("test");
    index.add_documents(json!([{ "id": 1, "content": "foobar" }, { "id": 0, "content": "foobar" }, { "id": 3, "content": "foobar" }]), Some("id")).await;
    index.wait_task(0).await;
    let (_response, code) = index.delete_batch(vec![]).await;
    assert_eq!(code, 202, "{}", _response);

    let _update = index.wait_task(1).await;
    let (response, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    assert_eq!(code, 200);
    assert_eq!(response["results"].as_array().unwrap().len(), 3);
}

#[actix_rt::test]
async fn delete_document_by_filter() {
    let server = Server::new().await;
    let index = server.index("doggo");
    index.update_settings_filterable_attributes(json!(["color"])).await;
    index
        .add_documents(
            json!([
                { "id": 0, "color": "red" },
                { "id": 1, "color": "blue" },
                { "id": 2, "color": "blue" },
                { "id": 3 },
            ]),
            Some("id"),
        )
        .await;
    index.wait_task(1).await;
    let (response, code) =
        index.delete_document_by_filter(json!({ "filter": "color = blue"})).await;
    snapshot!(code, @"202 Accepted");
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]" }), @r###"
    {
      "taskUid": 2,
      "indexUid": "doggo",
      "status": "enqueued",
      "type": "documentDeletion",
      "enqueuedAt": "[date]"
    }
    "###);

    let response = index.wait_task(2).await;
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "uid": 2,
      "indexUid": "doggo",
      "status": "succeeded",
      "type": "documentDeletion",
      "canceledBy": null,
      "details": {
        "providedIds": 0,
        "deletedDocuments": 2,
        "originalFilter": "\"color = blue\""
      },
      "error": null,
      "duration": "[duration]",
      "enqueuedAt": "[date]",
      "startedAt": "[date]",
      "finishedAt": "[date]"
    }
    "###);

    let (documents, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(documents), @r###"
    {
      "results": [
        {
          "id": 0,
          "color": "red"
        },
        {
          "id": 3
        }
      ],
      "offset": 0,
      "limit": 20,
      "total": 2
    }
    "###);

    let (response, code) =
        index.delete_document_by_filter(json!({ "filter": "color NOT EXISTS"})).await;
    snapshot!(code, @"202 Accepted");
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "taskUid": 3,
      "indexUid": "doggo",
      "status": "enqueued",
      "type": "documentDeletion",
      "enqueuedAt": "[date]"
    }
    "###);

    let response = index.wait_task(3).await;
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "uid": 3,
      "indexUid": "doggo",
      "status": "succeeded",
      "type": "documentDeletion",
      "canceledBy": null,
      "details": {
        "providedIds": 0,
        "deletedDocuments": 1,
        "originalFilter": "\"color NOT EXISTS\""
      },
      "error": null,
      "duration": "[duration]",
      "enqueuedAt": "[date]",
      "startedAt": "[date]",
      "finishedAt": "[date]"
    }
    "###);

    let (documents, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(documents), @r###"
    {
      "results": [
        {
          "id": 0,
          "color": "red"
        }
      ],
      "offset": 0,
      "limit": 20,
      "total": 1
    }
    "###);
}

#[actix_rt::test]
async fn delete_document_by_complex_filter() {
    let server = Server::new().await;
    let index = server.index("doggo");
    index.update_settings_filterable_attributes(json!(["color"])).await;
    index
        .add_documents(
            json!([
                { "id": 0, "color": "red" },
                { "id": 1, "color": "blue" },
                { "id": 2, "color": "blue" },
                { "id": 3, "color": "green" },
                { "id": 4 },
            ]),
            Some("id"),
        )
        .await;
    index.wait_task(1).await;
    let (response, code) = index
        .delete_document_by_filter(
            json!({ "filter": ["color != red", "color != green", "color EXISTS"] }),
        )
        .await;
    snapshot!(code, @"202 Accepted");
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]" }), @r###"
    {
      "taskUid": 2,
      "indexUid": "doggo",
      "status": "enqueued",
      "type": "documentDeletion",
      "enqueuedAt": "[date]"
    }
    "###);

    let response = index.wait_task(2).await;
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "uid": 2,
      "indexUid": "doggo",
      "status": "succeeded",
      "type": "documentDeletion",
      "canceledBy": null,
      "details": {
        "providedIds": 0,
        "deletedDocuments": 2,
        "originalFilter": "[\"color != red\",\"color != green\",\"color EXISTS\"]"
      },
      "error": null,
      "duration": "[duration]",
      "enqueuedAt": "[date]",
      "startedAt": "[date]",
      "finishedAt": "[date]"
    }
    "###);

    let (documents, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(documents), @r###"
    {
      "results": [
        {
          "id": 0,
          "color": "red"
        },
        {
          "id": 3,
          "color": "green"
        },
        {
          "id": 4
        }
      ],
      "offset": 0,
      "limit": 20,
      "total": 3
    }
    "###);

    let (response, code) = index
        .delete_document_by_filter(json!({ "filter": [["color = green", "color NOT EXISTS"]] }))
        .await;
    snapshot!(code, @"202 Accepted");
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "taskUid": 3,
      "indexUid": "doggo",
      "status": "enqueued",
      "type": "documentDeletion",
      "enqueuedAt": "[date]"
    }
    "###);

    let response = index.wait_task(3).await;
    snapshot!(json_string!(response, { ".enqueuedAt" => "[date]", ".startedAt" => "[date]", ".finishedAt" => "[date]", ".duration" => "[duration]" }), @r###"
    {
      "uid": 3,
      "indexUid": "doggo",
      "status": "succeeded",
      "type": "documentDeletion",
      "canceledBy": null,
      "details": {
        "providedIds": 0,
        "deletedDocuments": 4,
        "originalFilter": "[[\"color = green\",\"color NOT EXISTS\"]]"
      },
      "error": null,
      "duration": "[duration]",
      "enqueuedAt": "[date]",
      "startedAt": "[date]",
      "finishedAt": "[date]"
    }
    "###);

    let (documents, code) = index.get_all_documents(GetAllDocumentsOptions::default()).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(documents), @r###"
    {
      "results": [
        {
          "id": 0,
          "color": "red"
        }
      ],
      "offset": 0,
      "limit": 20,
      "total": 1
    }
    "###);
}
