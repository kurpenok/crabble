use bytes::Bytes;

use crabble::broker::Broker;

#[tokio::test]
async fn test_publish_subscribe() {
    let broker = Broker::new();
    broker
        .create_channel("test".to_string())
        .await
        .expect("Channel creation error");

    let (_id1, mut rx1) = broker.subscribe("test").await.expect("Subscribe error");
    let (_id2, mut rx2) = broker.subscribe("test").await.expect("Subscribe error");

    let test_msg = Bytes::from("hello");
    broker
        .publish("test", test_msg.clone())
        .await
        .expect("Publish error");

    let received1 = rx1.recv().await.expect("No message received");
    let received2 = rx2.recv().await.expect("No message received");
    assert_eq!(received1, test_msg);
    assert_eq!(received2, test_msg);
}

#[tokio::test]
async fn test_unsubscribe() {
    let broker = Broker::new();
    broker
        .create_channel("test".to_string())
        .await
        .expect("Channel creation error");
    let (sub_id, mut rx) = broker.subscribe("test").await.expect("Subscribe error");

    broker
        .unsubscribe("test", sub_id)
        .await
        .expect("Unsubscribe error");

    let test_msg = Bytes::from("world");
    broker
        .publish("test", test_msg)
        .await
        .expect("Publish error");

    let res = rx.recv().await;
    assert!(res.is_none(), "Message received after unsubscribing");
}
