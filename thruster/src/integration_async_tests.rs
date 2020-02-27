use std::boxed::Box;

use thruster::testing;
use thruster::middleware::{cookies, query_params};
use thruster::proc::{async_middleware, middleware_fn};
use thruster::BasicContext;
use thruster::{App, BasicContext as Ctx, MiddlewareNext, MiddlewareReturnValue, Request};
use tokio::runtime::Runtime;

#[middleware_fn]
async fn test_fn_1(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
    let body = &context.params.get("id").unwrap().clone();

    context.body(body);

    context
}

#[middleware_fn]
async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
    context.body("404");

    context
}

#[test]
fn it_should_correctly_404_if_no_param_is_given() {
    let mut app = App::<Request, Ctx>::new_basic();

    app.get("/test/:id", async_middleware!(Ctx, [test_fn_1]));
    app.set404(async_middleware!(Ctx, [test_fn_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "404");
    });
}

#[test]
fn it_should_execute_all_middlware_with_a_given_request() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app.use_middleware(
        "/",
        async_middleware!(BasicContext, [query_params::query_params]),
    );
    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_correctly_differentiate_wildcards_and_valid_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_fn_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("2");
        context
    };

    app.get("/", async_middleware!(BasicContext, [test_fn_1]));
    app.get("/*", async_middleware!(BasicContext, [test_fn_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_handle_query_parameters() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.query_params.get("hello").unwrap().clone();

        context.body(body);
        context
    };

    app.use_middleware(
        "/",
        async_middleware!(BasicContext, [query_params::query_params]),
    );
    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test?hello=world").await;

        assert!(response.body == "world");
    });
}

#[test]
fn it_should_handle_cookies() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = match context.cookies.get(0) {
            Some(cookie) => {
                assert!(cookie.options.same_site.is_some());
                assert!(cookie.options.same_site.as_ref().unwrap() == &cookies::SameSite::Strict);
                cookie.value.clone()
            }
            None => "".to_owned(),
        };

        context.body(&body);
        context
    };

    app.use_middleware("/", async_middleware!(BasicContext, [cookies::cookies]));
    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::request(
            &app,
            "GET",
            "/test",
            &vec![("Set-Cookie", "Hello=World; SameSite=Strict")],
            "",
        )
        .await;

        assert!(response.body == "World");
    });
}

#[test]
fn it_should_execute_all_middlware_with_a_given_request_with_params() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    app.get("/test/:id", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test/123").await;

        assert!(response.body == "123");
    });
}

#[test]
fn it_should_execute_all_middlware_with_a_given_request_with_params_in_a_subapp() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    app1.get("/:id", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/test/123").await;

        assert!(response.body == "123");
    });
}

#[test]
fn it_should_correctly_parse_params_in_subapps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    app1.get("/:id", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/test/123").await;

        assert!(response.body == "123");
    });
}

#[test]
fn it_should_match_as_far_as_possible_in_a_subapp() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("-1");
        context
    }

    app1.get("/", async_middleware!(BasicContext, [test_fn_2]));
    app1.get("/:id", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/test/123").await;

        assert!(response.body == "123");
    });
}

#[test]
fn it_should_trim_trailing_slashes() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("-1");
        context
    }

    app1.get("/:id", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);
    app2.set404(async_middleware!(BasicContext, [test_fn_2]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/test/").await;

        assert!(response.body == "-1");
    });
}

#[test]
fn it_should_trim_trailing_slashes_after_params() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = &context.params.get("id").unwrap().clone();

        context.body(body);
        context
    };

    app.get("/test/:id", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test/1/").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let existing_body = context.get_body().clone();

        context.body(&format!("{}{}", existing_body, "1"));
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let existing_body = context.get_body().clone();

        context.body(&format!("{}{}", existing_body, "2"));
        context
    };

    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));
    app.post("/test", async_middleware!(BasicContext, [test_fn_2]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let existing_body = context.get_body().clone();

        context.body(&format!("{}{}", existing_body, "1"));
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let existing_body = context.get_body().clone();

        context.body(&format!("{}{}", existing_body, "2"));

        let mut context_with_body = next(context).await;
        let existing_body = context_with_body.get_body().clone();
        context_with_body.body(&format!("{}{}", existing_body, "2"));

        context_with_body
    };

    app.get(
        "/test",
        async_middleware!(BasicContext, [test_fn_2, test_fn_1]),
    );

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "212");
    });
}

#[test]
fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("Hello world");
        context
    };

    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "Hello world");
    });
}

#[test]
fn it_should_first_run_use_then_methods() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn method_agnostic(
        mut context: BasicContext,
        next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("agnostic");
        let updated_context = next(context).await;

        updated_context
    }

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        let body = context.get_body().clone();

        context.body(&format!("{}-1", body));
        context
    };

    app.use_middleware("/", async_middleware!(BasicContext, [method_agnostic]));
    app.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/test").await;

        assert!(response.body == "agnostic-1");
    });
}

#[test]
fn it_should_be_able_to_correctly_route_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app1.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/test").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_be_able_to_correctly_route_sub_apps_with_wildcards() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app1.get("/*", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/a").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_be_able_to_correctly_prefix_route_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app1.get("/test", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/sub", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/sub/test").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_be_able_to_correctly_prefix_the_root_of_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app1.get("/", async_middleware!(BasicContext, [test_fn_1]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/sub", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/sub").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_not_found_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app.get("/", async_middleware!(BasicContext, [test_fn_1]));
    app.set404(async_middleware!(BasicContext, [test_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/not_found").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_not_found_at_the_root() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app.get("/a", async_middleware!(BasicContext, [test_fn_1]));
    app.set404(async_middleware!(BasicContext, [test_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_deep_not_found_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app.get("/a/b", async_middleware!(BasicContext, [test_fn_1]));
    app.set404(async_middleware!(BasicContext, [test_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/a/not_found/").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app.get("/a/:b/c", async_middleware!(BasicContext, [test_fn_1]));
    app.set404(async_middleware!(BasicContext, [test_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/a/1/d").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes_with_extra_pieces(
) {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app.get("/a/:b/c", async_middleware!(BasicContext, [test_fn_1]));
    app.set404(async_middleware!(BasicContext, [test_404]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/a/1/d/e/f/g").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_root_route() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();
    let mut app3 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_404(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("not found");
        context
    };

    app1.get("/", async_middleware!(BasicContext, [test_fn_1]));
    app2.get("*", async_middleware!(BasicContext, [test_404]));
    app3.use_sub_app("/", app2);
    app3.use_sub_app("/a", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app3, "/a/1/d").await;

        assert!(response.body == "not found");
    });
}

#[test]
fn it_should_handle_routes_without_leading_slash() {
    let mut app = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app.get("*", async_middleware!(BasicContext, [test_fn_1]));

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app, "/a/1/d/e/f/g").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_handle_routes_within_a_subapp_without_leading_slash() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    app1.get("*", async_middleware!(BasicContext, [test_fn_1]));
    app2.use_sub_app("/", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/a/1/d/e/f/g").await;

        assert!(response.body == "1");
    });
}

#[test]
fn it_should_handle_multiple_subapps_with_wildcards() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();
    let mut app3 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("2");
        context
    };

    app1.get("/*", async_middleware!(BasicContext, [test_fn_1]));
    app2.get("/*", async_middleware!(BasicContext, [test_fn_2]));
    app3.use_sub_app("/", app1);
    app3.use_sub_app("/a/b", app2);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app3, "/a/1/d/e/f/g").await;

        assert!(response.body == "1");
    });
}


#[test]
fn it_should_prefer_specificity_to_ambiguity() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    #[middleware_fn]
    async fn test_fn_1(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("1");
        context
    };

    #[middleware_fn]
    async fn test_fn_2(
        mut context: BasicContext,
        _next: MiddlewareNext<BasicContext>,
    ) -> BasicContext {
        context.body("2");
        context
    };

    app1.get("/:id", async_middleware!(BasicContext, [test_fn_1]));
    app1.get("/order", async_middleware!(BasicContext, [test_fn_2]));

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/b", app1);

    let _ = Runtime::new().unwrap().block_on(async {
        let response = testing::get(&app2, "/b/order").await;

        assert!(response.body == "2");
    });
}
