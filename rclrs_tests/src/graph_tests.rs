use std::panic;

use rclrs::{Context, Node, RclrsError, QOS_PROFILE_SYSTEM_DEFAULT};

use test_msgs::msg;

fn run_with_test_nodes<F>(test_fn: F) -> Result<(), RclrsError>
where
    F: FnOnce() -> Result<(), RclrsError> + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| test_fn());

    result.unwrap()
}

#[test]
fn test_publishers() {
    let context = Context::new([]).unwrap();
    let node_name = "test_publishers";
    let node = Node::new(&context, node_name).unwrap();

    let _publisher_basic_types = node
        .create_publisher::<msg::BasicTypes>("test", QOS_PROFILE_SYSTEM_DEFAULT)
        .expect("Failed to create String publisher");

    let _publisher_empty = node
        .create_publisher::<msg::Empty>("test", QOS_PROFILE_SYSTEM_DEFAULT)
        .expect("Failed to create Bool publisher");

    let publisher_names_and_types = node
        .get_publisher_names_and_types_by_node(&node.name(), "/")
        .expect("Failed to get names and types of the publisher");

    let (name, types) = publisher_names_and_types.get_key_value("/test").unwrap();

    if node.count_publishers("/test").unwrap() != 2 {
        println!("{:?}", publisher_names_and_types);
    }

    assert_eq!(name, "/test");
    assert_eq!(node.count_publishers("/test").unwrap(), 2);
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));
}

#[test]
fn test_subscriptions() {
    let context = Context::new([]).unwrap();
    let node_name = "test_subscriptions";
    let mut node = Node::new(&context, node_name).unwrap();

    let _subscription_basic_types = node.create_subscription::<msg::BasicTypes, _>(
        "test",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    );

    let _subscription_empty = node
        .create_subscription::<msg::Empty, _>("test", QOS_PROFILE_SYSTEM_DEFAULT, |_msg| {})
        .expect("Failed to create Bool subscription");

    let subscription_names_and_types = node
        .get_subscription_names_and_types_by_node(&node.name(), "/")
        .expect("Failed to get names and types of the subscription");

    let (name, types) = subscription_names_and_types.get_key_value("/test").unwrap();

    assert_eq!(name, "/test");
    assert_eq!(node.count_subscriptions("/test").unwrap(), 2);
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));
}

#[test]
fn test_topic_names_and_types() {
    let context = Context::new([]).unwrap();
    let node_name = "test_topic_names_and_types";
    let mut node = Node::new(&context, node_name).unwrap();

    let _publisher_basic_types = node
        .create_publisher::<msg::BasicTypes>("test", QOS_PROFILE_SYSTEM_DEFAULT)
        .expect("Failed to create String publisher");

    let _subscription_bool = node
        .create_subscription::<msg::Empty, _>("test", QOS_PROFILE_SYSTEM_DEFAULT, |_msg| {})
        .expect("Failed to create Bool subscription");

    let topic_names_and_types = node
        .get_topic_names_and_types()
        .expect("Failed to get names and types of the subscription");

    let (name, types) = topic_names_and_types.get_key_value("/test").unwrap();

    assert_eq!(name, "/test");
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));
}
