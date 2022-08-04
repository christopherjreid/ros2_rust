use rclrs::{
    Context, Node, NodeBuilder, RclrsError, TopicNamesAndTypes, QOS_PROFILE_SYSTEM_DEFAULT,
};

use test_msgs::{msg, srv};

struct TestGraph {
    node1: Node,
    node2: Node,
}

fn construct_test_graph(namespace: &str) -> Result<TestGraph, RclrsError> {
    let context = Context::new([])?;
    Ok(TestGraph {
        node1: NodeBuilder::new(&context, "graph_test_node_1")
            .namespace(namespace)
            .build()?,
        node2: NodeBuilder::new(&context, "graph_test_node_2")
            .namespace(namespace)
            .build()?,
    })
}

#[test]
fn test_publishers() -> Result<(), RclrsError> {
    let graph = construct_test_graph("test_publishers_graph")?;

    let _node_1_empty_publisher = graph
        .node1
        .create_publisher::<msg::Empty>("graph_test_topic_1", QOS_PROFILE_SYSTEM_DEFAULT)?;
    let _node_1_basic_types_publisher = graph
        .node1
        .create_publisher::<msg::BasicTypes>("graph_test_topic_2", QOS_PROFILE_SYSTEM_DEFAULT)?;
    let _node_2_default_publisher = graph
        .node2
        .create_publisher::<msg::Defaults>("graph_test_topic_3", QOS_PROFILE_SYSTEM_DEFAULT)?;

    assert_eq!(
        graph
            .node1
            .count_publishers(&(graph.node1.namespace() + "/graph_test_topic_1"))?,
        1
    );
    assert_eq!(
        graph
            .node1
            .count_publishers(&(graph.node1.namespace() + "/graph_test_topic_2"))?,
        1
    );
    let node_1_publisher_names_and_types = graph
        .node1
        .get_publisher_names_and_types_by_node(&graph.node1.name(), &graph.node1.namespace())?;

    let types = node_1_publisher_names_and_types
        .get(&(graph.node1.namespace() + "/graph_test_topic_1"))
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));

    let types = node_1_publisher_names_and_types
        .get(&(graph.node1.namespace() + "/graph_test_topic_2"))
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));

    let node_2_publisher_names_and_types = graph
        .node2
        .get_publisher_names_and_types_by_node(&graph.node2.name(), &graph.node2.namespace())?;

    let types = node_2_publisher_names_and_types
        .get(&(graph.node2.namespace() + "/graph_test_topic_3"))
        .unwrap();
    assert_eq!(
        graph
            .node2
            .count_publishers(&(graph.node2.namespace() + "/graph_test_topic_3"))?,
        1
    );
    assert!(types.contains(&"test_msgs/msg/Defaults".to_string()));

    Ok(())
}

#[test]
fn test_subscriptions() -> Result<(), RclrsError> {
    let mut graph = construct_test_graph("test_subscriptions_graph")?;

    let _node_1_defaults_subscription = graph.node1.create_subscription::<msg::Defaults, _>(
        "graph_test_topic_3",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;
    let _node_2_empty_subscription = graph.node2.create_subscription::<msg::Empty, _>(
        "graph_test_topic_1",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;
    let _node_2_basic_types_subscription = graph.node2.create_subscription::<msg::BasicTypes, _>(
        "graph_test_topic_2",
        QOS_PROFILE_SYSTEM_DEFAULT,
        |_msg| {},
    )?;

    let node_1_subscription_names_and_types = graph
        .node1
        .get_subscription_names_and_types_by_node(&graph.node1.name(), &graph.node1.namespace())?;

    let types = node_1_subscription_names_and_types
        .get(&(graph.node2.namespace() + "/graph_test_topic_3"))
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Defaults".to_string()));

    assert_eq!(
        graph
            .node2
            .count_subscriptions(&(graph.node2.namespace() + "/graph_test_topic_1"))?,
        1
    );
    assert_eq!(
        graph
            .node2
            .count_subscriptions(&(graph.node2.namespace() + "/graph_test_topic_2"))?,
        1
    );

    let node_2_subscription_names_and_types = graph
        .node2
        .get_subscription_names_and_types_by_node(&graph.node2.name(), &graph.node2.namespace())?;

    let types = node_2_subscription_names_and_types
        .get(&(graph.node1.namespace() + "/graph_test_topic_1"))
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/Empty".to_string()));

    let types = node_2_subscription_names_and_types
        .get(&(graph.node2.namespace() + "/graph_test_topic_2"))
        .unwrap();
    assert!(types.contains(&"test_msgs/msg/BasicTypes".to_string()));

    Ok(())
}

#[test]
fn test_graph() -> Result<(), RclrsError> {
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

    check_topic_names_and_types(&node1)?;
    check_services(&node1)?;
    check_clients(&node2)?;

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
