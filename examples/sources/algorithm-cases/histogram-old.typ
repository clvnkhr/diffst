= Router Module

```rust
pub fn install_routes(app: &mut Router) {
    app.get("/health", health);
    app.get("/users", list_users);
    app.post("/users", create_user);
    app.get("/projects", list_projects);
    app.post("/projects", create_project);
}

fn health(req: Request) -> Response {
    let span = trace("health");
    let result = service::health(req);
    span.finish();
    result
}

fn list_users(req: Request) -> Response {
    let span = trace("list_users");
    let result = service::list_users(req);
    span.finish();
    result
}

fn create_user(req: Request) -> Response {
    let span = trace("create_user");
    let result = service::create_user(req);
    span.finish();
    result
}

fn list_projects(req: Request) -> Response {
    let span = trace("list_projects");
    let result = service::list_projects(req);
    span.finish();
    result
}

fn create_project(req: Request) -> Response {
    let span = trace("create_project");
    let result = service::create_project(req);
    span.finish();
    result
}
```

