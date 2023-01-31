use super::{payment::MockPayment, rocket, runner::MockRunner, Db};
use mockall::predicate::{always, eq};
use rocket::{
    http::{Header, Status},
    local::blocking::Client,
};
use rocket_db_pools::{sqlx::Executor, Database};

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(MockPayment, MockRunner) -> (),
{
    use std::env;
    if env::var("MANJALIOF_BACKEND_TOKEN").is_err() {
        env::set_var("MANJALIOF_BACKEND_TOKEN", "somestrongtoken");
    }

    reset_db();
    let payment = MockPayment::new();
    let runner = MockRunner::new();
    test(payment, runner);
}

fn reset_db() {
    let client = Client::untracked(rocket(MockPayment::new(), MockRunner::new())).unwrap();
    let db = Db::fetch(client.rocket()).unwrap();
    rocket::async_test(async move {
        db.execute("DELETE FROM transactions").await.unwrap();
    });
}

#[test]
fn create_payment_should_fail_when_notauthorized() {
    run_test(|payment, runner| {
        let client = Client::untracked(rocket(payment, runner)).unwrap();
        assert_eq!(
            client.get("/create_payment").dispatch().status(),
            Status::NotFound
        );
        assert_eq!(
            client.post("/create_payment").dispatch().status(),
            Status::Unauthorized
        );
        assert_eq!(
            client
                .post("/create_payment")
                .header(Header::new("auth_token", "wrong_token"))
                .dispatch()
                .status(),
            Status::Unauthorized
        );
    });
}

#[test]
fn create_payment_should_fail_when_parameters_are_wrong() {
    run_test(|payment, runner| {
        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let mut req = client.post("/create_payment");
        req.add_header(Header::new("auth_token", "somestrongtoken"));

        assert_eq!(
            req.clone().body(r#""#).dispatch().status(),
            Status::BadRequest
        );
        assert_eq!(
            req.clone().body(r#"{}"#).dispatch().status(),
            Status::UnprocessableEntity
        );
        assert_eq!(
            req.clone()
                .body(r#"{ "clients": "wtf_this_has_to_be_array"}"#)
                .dispatch()
                .status(),
            Status::UnprocessableEntity
        );

        let res = req
            .body(r#"{ "clients": [], "reffer": "arian" }"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            r#"{"success":false,"message":"at least provide one client"}"#
        );
    });
}

#[test]
fn create_payment_should_fail_when_runner_cannot_validate() {
    run_test(|payment, mut runner| {
        runner
            .expect_validate_clients()
            .with(eq(vec!["arian".to_string()]))
            .returning(|_| Err("something wrong".to_string()));

        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let req = client
            .post("/create_payment")
            .header(Header::new("auth_token", "somestrongtoken"));
        let res = req.body(r#"{ "clients": ["arian"] }"#).dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            r#"{"success":false,"message":"cannot validate clients: something wrong"}"#
        );
    });
}

#[test]
fn create_payment_should_fail_when_runner_cannot_request_payment() {
    run_test(|mut payment, mut runner| {
        runner.expect_validate_clients().returning(|_| Ok(()));
        payment
            .expect_request_payment_authority()
            .with(eq("arian"), always())
            .returning(|_, _| Err("some error".to_string()));

        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let req = client
            .post("/create_payment")
            .header(Header::new("auth_token", "somestrongtoken"));
        let res = req.body(r#"{ "clients": ["arian"] }"#).dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            r#"{"success":false,"message":"cannot request payment: some error"}"#
        );
    });
}

#[test]
fn create_payment() {
    run_test(|mut payment, mut runner| {
        runner
            .expect_validate_clients()
            .with(eq(vec!["someone".to_string(), "anotherone".to_string()]))
            .returning(|_| Ok(()));

        let authority = generate_random_authority();
        let authority_clone = authority.clone();
        payment
            .expect_request_payment_authority()
            .with(eq("someone,anotherone"), always())
            .returning(move |_, _| Ok(authority_clone.clone()));

        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let req = client
            .post("/create_payment")
            .header(Header::new("auth_token", "somestrongtoken"));
        let res = req
            .body(r#"{ "clients": ["someone", "anotherone"] }"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            format!(r#"{{"success":true,"message":"{authority}"}}"#)
        );
    });
}

fn generate_random_authority() -> String {
    let random_number: String = rand::random::<u32>().to_string();
    let zeros = "0".repeat(35 - random_number.len());
    format!("A{zeros}{random_number}")
}

#[test]
fn create_and_verify_payment() {
    run_test(|mut payment, mut runner| {
        runner
            .expect_validate_clients()
            .with(eq(vec!["someone".to_string(), "anotherone".to_string()]))
            .times(1)
            .returning(|_| Ok(()));

        payment
            .expect_request_payment_authority()
            .with(eq("someone,anotherone"), always())
            .times(1)
            .returning(move |_, _| Ok("generated_authority".to_string()));

        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let req = client
            .post("/create_payment")
            .header(Header::new("auth_token", "somestrongtoken"));
        let res = req
            .body(r#"{ "clients": ["someone", "anotherone"] }"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            r#"{"success":true,"message":"generated_authority"}"#
        );

        let mut payment = MockPayment::new();
        payment
            .expect_verify()
            .with(eq("generated_authority"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        let mut runner = MockRunner::new();
        runner
            .expect_make_client_paid()
            .with(eq("someone"))
            .times(1)
            .returning(|_| Ok(()));
        runner
            .expect_make_client_paid()
            .with(eq("anotherone"))
            .times(1)
            .returning(|_| Ok(()));

        let client = Client::untracked(rocket(payment, runner)).unwrap();
        let res = client
            .post("/verify_payment")
            .body(r#"{ "authority": "generated_authority"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.into_string().unwrap(),
            r#"{"success":true,"message":""}"#
        );
    });
}
