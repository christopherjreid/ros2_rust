use rclrs::{Context, Node, RclrsError, TopicNamesAndTypes, QOS_PROFILE_SYSTEM_DEFAULT};

use test_msgs::{msg, srv};

#[test]
fn foo() -> Result<(), RclrsError> {
    let context = Context::new([])?;
    let mut node1 = Node::new(&context, "graph_test_node_1")?;
    let mut node2 = Node::new(&context, "graph_test_node_2")?;

    let _node_1_empty_publisher =
        node1.create_publisher::<msg::Empty>("graph_test_topic_1", QOS_PROFILE_SYSTEM_DEFAULT)?;
    let _node_1_basic_types_publisher = node1
        .create_publisher::<msg::BasicTypes>("graph_test_topic_2", QOS_PROFILE_SYSTEM_DEFAULT)?;
    let _node_1_defaults_subscription = node1.create_subscription::<msg::Defaults, _>(
        "graph_test_topic_3",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;
    let _node_1_empty_service =
        node1.create_service::<srv::Empty, _>("graph_test_topic_4", |_request_id, _srv| {
            srv::Empty_Response {
                structure_needs_at_least_one_member: 0,
            }
        })?;

    let _node_2_default_publisher = node2
        .create_publisher::<msg::Defaults>("graph_test_topic_3", QOS_PROFILE_SYSTEM_DEFAULT)?;
    let _node_2_empty_subscription = node2.create_subscription::<msg::Empty, _>(
        "graph_test_topic_1",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;
    let _node_2_basic_types_subscription = node2.create_subscription::<msg::BasicTypes, _>(
        "graph_test_topic_2",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;
    let _node_2_empty_client = node2.create_client::<srv::Empty>("graph_test_topic_4")?;

    check_publishers(&node1, &node2)?;
    check_subscriptions(&node1, &node2)?;
    check_topic_names_and_types(&node1)?;
    check_services(&node1)?;
    check_clients(&node2)?;

    Ok(())
}

fn check_publishers(node1: &Node, node2: &Node) -> Result<(), RclrsError> {
    assert_eq!(node1.count_publishers("/graph_test_topic_1")?, 1);
    assert_eq!(node1.count_publishers("/graph_test_topic_2")?, 1);
    let node_1_publisher_names_and_types =
        node1.get_publisher_names_and_types_by_node("graph_test_node_1", "/")?;

    let types = node_1_publisher_names_and_types
        .get("/graph_test_topic_1")
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));

    let types = node_1_publisher_names_and_types
        .get("/graph_test_topic_2")
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));

    let node_2_publisher_names_and_types =
        node2.get_publisher_names_and_types_by_node("graph_test_node_2", "/")?;

    let types = node_2_publisher_names_and_types
        .get("/graph_test_topic_3")
        .unwrap();
    assert_eq!(node2.count_publishers("/graph_test_topic_3")?, 1);
    assert!(types.contains(&"test_msgs/msg/Defaults".to_string()));

    Ok(())
}

fn check_subscriptions(node1: &Node, node2: &Node) -> Result<(), RclrsError> {
    assert_eq!(node1.count_subscriptions("/graph_test_topic_3")?, 1);
    let node_1_subscription_names_and_types =
        node1.get_subscription_names_and_types_by_node("graph_test_node_1", "/")?;

    let types = node_1_subscription_names_and_types
        .get("/graph_test_topic_3")
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Defaults".to_string()));

    assert_eq!(node2.count_subscriptions("/graph_test_topic_1")?, 1);
    assert_eq!(node2.count_subscriptions("/graph_test_topic_2")?, 1);

    let node_2_subscription_names_and_types =
        node2.get_subscription_names_and_types_by_node("graph_test_node_2", "/")?;

    let types = node_2_subscription_names_and_types
        .get("/graph_test_topic_1")
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));

    let types = node_2_subscription_names_and_types
        .get("/graph_test_topic_2")
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));

    Ok(())
}

fn check_topic_names_and_types(node: &Node) -> Result<(), RclrsError> {
    let topic_names_and_types = node.get_topic_names_and_types()?;

    let types = topic_names_and_types.get("/graph_test_topic_1").unwrap();
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));
    let types = topic_names_and_types.get("/graph_test_topic_2").unwrap();
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));

    let types = topic_names_and_types.get("/graph_test_topic_3").unwrap();
    assert!(types.contains(&"test_msgs/msg/Defaults".to_string()));

    Ok(())
}

fn check_services(node: &Node) -> Result<(), RclrsError> {
    let check_names_and_types = |names_and_types: TopicNamesAndTypes| {
        let types = names_and_types.get("/graph_test_topic_4").unwrap();
        assert!(types.contains(&"test_msgs/srv/Empty".to_string()));
    };

    let service_names_and_types = node.get_service_names_and_types()?;
    check_names_and_types(service_names_and_types);

    let service_names_and_types =
        node.get_service_names_and_types_by_node(&node.name(), &node.namespace())?;
    check_names_and_types(service_names_and_types);

    Ok(())
}

fn check_clients(node: &Node) -> Result<(), RclrsError> {
    let client_names_and_types =
        node.get_client_names_and_types_by_node(&node.name(), &node.namespace())?;
    let types = client_names_and_types.get("/graph_test_topic_4").unwrap();
    assert!(types.contains(&"test_msgs/srv/Empty".to_string()));

    Ok(())
}
