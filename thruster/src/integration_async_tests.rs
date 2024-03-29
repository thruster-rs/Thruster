// use std::boxed::Box;

// use thruster::middleware::{cookies, query_params};
// use thruster::testing;
// use thruster::BasicContext;
// use thruster::{
//     m, middleware_fn, App, BasicContext as Ctx, MiddlewareNext, MiddlewareResult,
//     Request,
// };
// use tokio::runtime::Runtime;

// #[middleware_fn]
// async fn test_fn_1(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
//     let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//     context.body(body);

//     Ok(context)
// }

// #[middleware_fn]
// async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
//     context.body("404");

//     Ok(context)
// }

// #[test]
// fn it_should_correctly_404_if_no_param_is_given() {
//     let mut app = App::<Request, Ctx, ()>::new_basic();

//     app.get("/test/:id", m![test_fn_1]);
//     app.set404(m![test_fn_404]);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "404");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app.use_middleware(
//         "/",
//         m!(BasicContext, [query_params::query_params]),
//     );
//     app.get("/test", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_once() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body(&format!("{}{}", context.get_body(), "1"));

//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_middleware(
//         mut context: BasicContext,
//         next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body(&format!("{}{}", context.get_body(), "0"));

//         let mut context = next(context).await.unwrap();

//         context.body(&format!("{}{}", context.get_body(), "0"));

//         Ok(context)
//     };

//     app.get("/test", m!(BasicContext, [test_fn_1]));
//     app.use_middleware("/", m!(BasicContext, [test_fn_middleware]));

//     // println!("app: {}", app._route_parser.route_tree.root_node.tree_string(""));
//     // for (route, middleware, is_terminal) in app._route_parser.route_tree.root_node.get_route_list() {
//     //     println!("{}: {}", route, middleware.len());
//     // }

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "010");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_with_prefix() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("nope");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_middleware_1(
//         context: BasicContext,
//         next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let _ = next(context).await;
//         let mut context = Ctx::new();
//         context.body("1");
//         Ok(context)
//     };

//     app.use_middleware(
//         "/test",
//         m!(BasicContext, [test_middleware_1]),
//     );
//     app.get("/test/:key", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test/a").await;

//         println!("response.body: {}", response.body);
//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_correctly_differentiate_wildcards_and_valid_routes() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("2");
//         Ok(context)
//     };

//     app.get("/", m!(BasicContext, [test_fn_1]));
//     app.get("/*", m!(BasicContext, [test_fn_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_handle_query_parameters() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context
//             .query_params
//             .as_ref()
//             .unwrap()
//             .get("hello")
//             .unwrap()
//             .clone();

//         context.body(body);
//         Ok(context)
//     };

//     app.use_middleware(
//         "/",
//         m!(BasicContext, [query_params::query_params]),
//     );
//     app.get("/test", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test?hello=world").await;

//         assert!(response.body == "world");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_with_params() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     app.get("/test/:id", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test/123").await;

//         assert!(response.body == "123");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_with_params_in_a_subapp() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     app1.get("/:id", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/test", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/test/123").await;

//         assert!(response.body == "123");
//     });
// }

// #[test]
// fn it_should_correctly_parse_params_in_subapps() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     app1.get("/:id", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/test", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/test/123").await;

//         assert!(response.body == "123");
//     });
// }

// #[test]
// fn it_should_match_as_far_as_possible_in_a_subapp() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("-1");
//         Ok(context)
//     }

//     app1.get("/", m!(BasicContext, [test_fn_2]));
//     app1.get("/:id", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/test", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/test/123").await;

//         assert!(response.body == "123");
//     });
// }

// #[test]
// fn it_should_trim_trailing_slashes() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("-1");
//         Ok(context)
//     }

//     app1.get("/:id", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/test", app1);
//     app2.set404(m!(BasicContext, [test_fn_2]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/test/").await;

//         assert!(response.body == "-1");
//     });
// }

// #[test]
// fn it_should_trim_trailing_slashes_after_params() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = &context.params.as_ref().unwrap().get("id").unwrap().clone();

//         context.body(body);
//         Ok(context)
//     };

//     app.get("/test/:id", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test/1/").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let existing_body = context.get_body().clone();

//         context.body(&format!("{}{}", existing_body, "1"));
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let existing_body = context.get_body().clone();

//         context.body(&format!("{}{}", existing_body, "2"));
//         Ok(context)
//     };

//     app.get("/test", m!(BasicContext, [test_fn_1]));
//     app.post("/test", m!(BasicContext, [test_fn_2]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let existing_body = context.get_body().clone();

//         context.body(&format!("{}{}", existing_body, "1"));
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let existing_body = context.get_body().clone();

//         context.body(&format!("{}{}", existing_body, "2"));

//         let mut context_with_body = next(context).await?;
//         let existing_body = context_with_body.get_body().clone();
//         context_with_body.body(&format!("{}{}", existing_body, "2"));

//         Ok(context_with_body)
//     };

//     app.get(
//         "/test",
//         m!(BasicContext, [test_fn_2, test_fn_1]),
//     );

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "212");
//     });
// }

// #[test]
// fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("Hello world");
//         Ok(context)
//     };

//     app.get("/test", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "Hello world");
//     });
// }

// #[test]
// fn it_should_first_run_use_then_methods() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn method_agnostic(
//         mut context: BasicContext,
//         next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("agnostic");
//         let updated_context = next(context).await?;

//         Ok(updated_context)
//     }

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         let body = context.get_body().clone();

//         context.body(&format!("{}-1", body));
//         Ok(context)
//     };

//     app.use_middleware("/", m!(BasicContext, [method_agnostic]));
//     app.get("/test", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/test").await;

//         assert!(response.body == "agnostic-1");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_route_sub_apps() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app1.get("/test", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/test").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_route_sub_apps_with_wildcards() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app1.get("/*", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/a").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_prefix_route_sub_apps() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app1.get("/test", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/sub", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/sub/test").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_prefix_the_root_of_sub_apps() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app1.get("/", m!(BasicContext, [test_fn_1]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/sub", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/sub").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_not_found_routes() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app.get("/", m!(BasicContext, [test_fn_1]));
//     app.set404(m!(BasicContext, [test_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/not_found").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_not_found_at_the_root() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app.get("/a", m!(BasicContext, [test_fn_1]));
//     app.set404(m!(BasicContext, [test_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_deep_not_found_routes() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app.get("/a/b", m!(BasicContext, [test_fn_1]));
//     app.set404(m!(BasicContext, [test_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/a/not_found/").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app.get("/a/:b/c", m!(BasicContext, [test_fn_1]));
//     app.set404(m!(BasicContext, [test_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/a/1/d").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes_with_extra_pieces(
// ) {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app.get("/a/:b/c", m!(BasicContext, [test_fn_1]));
//     app.set404(m!(BasicContext, [test_404]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/a/1/d/e/f/g").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_root_route() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();
//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     let mut app3 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_404(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("not found");
//         Ok(context)
//     };

//     app1.get("/", m!(BasicContext, [test_fn_1]));
//     app2.get("*", m!(BasicContext, [test_404]));
//     app3.use_sub_app("/", app2);
//     app3.use_sub_app("/a", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app3, "/a/1/d").await;

//         assert!(response.body == "not found");
//     });
// }

// #[test]
// fn it_should_handle_routes_without_leading_slash() {
//     let mut app = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app.get("*", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app, "/a/1/d/e/f/g").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_handle_routes_within_a_subapp_without_leading_slash() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();
//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     app1.get("*", m!(BasicContext, [test_fn_1]));
//     app2.use_sub_app("/", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/a/1/d/e/f/g").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_handle_multiple_subapps_with_wildcards() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();
//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     let mut app3 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("2");
//         Ok(context)
//     };

//     app1.get("/*", m!(BasicContext, [test_fn_1]));
//     app2.get("/*", m!(BasicContext, [test_fn_2]));
//     app3.use_sub_app("/", app1);
//     app3.use_sub_app("/a/b", app2);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app3, "/a/1/d/e/f/g").await;

//         assert!(response.body == "1");
//     });
// }

// #[test]
// fn it_should_prefer_specificity_to_ambiguity() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("2");
//         Ok(context)
//     };

//     app1.get("/:id", m!(BasicContext, [test_fn_1]));
//     app1.get("/order", m!(BasicContext, [test_fn_2]));

//     let mut app2 = App::<Request, BasicContext, ()>::new_basic();
//     app2.use_sub_app("/b", app1);

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app2, "/b/order").await;

//         assert!(response.body == "2");
//     });
// }

// #[test]
// fn it_should_prefer_nested_specificity_to_immediate_ambiguity() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("2");
//         Ok(context)
//     };

//     app1.get("/:id/d", m!(BasicContext, [test_fn_2]));
//     app1.get("/:id", m!(BasicContext, [test_fn_1]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app1, "/b/d").await;

//         println!("response.body: {}", response.body);
//         assert!(response.body == "2");
//     });
// }

// #[test]
// fn it_should_not_overwrite_wildcards_when_nesting() {
//     let mut app1 = App::<Request, BasicContext, ()>::new_basic();

//     #[middleware_fn]
//     async fn test_fn_1(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("1");
//         Ok(context)
//     };

//     #[middleware_fn]
//     async fn test_fn_2(
//         mut context: BasicContext,
//         _next: MiddlewareNext<BasicContext>,
//     ) -> MiddlewareResult<BasicContext> {
//         context.body("2");
//         Ok(context)
//     };

//     app1.get("/a/:id", m!(BasicContext, [test_fn_1]));
//     app1.get("/a/:id/d", m!(BasicContext, [test_fn_2]));

//     let _ = Runtime::new().unwrap().block_on(async {
//         let response = testing::get(&app1, "/a/b").await;

//         assert!(response.body == "1");
//     });
// }
