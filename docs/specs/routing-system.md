# RUX Routing System Design

## Overview

RUX provides a file-based routing system inspired by Next.js and Remix, with support for nested routes, loaders, actions, and universal routing (server + client).

## 1. File-Based Routing

### 1.1 Route Structure

Routes are defined by file structure.

```
src/
├── routes/
│   ├── index.rsx          → /
│   ├── about.rsx          → /about
│   ├── blog/
│   │   ├── index.rsx      → /blog
│   │   └── [slug].rsx     → /blog/:slug
│   └── users/
│       ├── index.rsx      → /users
│       └── [id]/
│           ├── index.rsx  → /users/:id
│           └── edit.rsx   → /users/:id/edit
```

### 1.2 Route Naming

Route files follow conventions.

- `index.rsx` → route path
- `[param].rsx` → dynamic segment
- `[...param].rsx` → catch-all
- `(group)/` → route group (not in URL)

### 1.3 Route Parameters

Accessing route parameters.

```rsx
// routes/blog/[slug].rsx
fn BlogPost() -> Element {
    let params = use_params();
    let slug = params.get("slug").unwrap();
    
    <div>
        <h1>Post: {slug}</h1>
    </div>
}
```

## 2. Nested Routing

### 2.1 Nested Routes

Nested route structure.

```
routes/
├── layout.rsx
└── dashboard/
    ├── layout.rsx
    ├── index.rsx
    └── settings/
        └── index.rsx
```

### 2.2 Layout Components

Layout components wrap child routes.

```rsx
// routes/layout.rsx
fn Layout(children: Element) -> Element {
    <div class="app">
        <Header />
        <main>
            {children}
        </main>
        <Footer />
    </div>
}

// routes/dashboard/layout.rsx
fn DashboardLayout(children: Element) -> Element {
    <div class="dashboard">
        <Sidebar />
        <div class="content">
            {children}
        </div>
    </div>
}
```

### 2.3 Outlet Component

Rendering nested routes.

```rsx
fn Layout(children: Element) -> Element {
    <div>
        <Header />
        <Outlet /> {/* Renders nested route */}
        <Footer />
    </div>
}
```

## 3. Loader Functions

### 3.1 Data Loading

Loading data before rendering.

```rsx
// routes/blog/[slug].rsx
async fn loader(params: RouteParams) -> Result<BlogPost> {
    let slug = params.get("slug")?;
    let post = fetch_blog_post(slug).await?;
    Ok(post)
}

fn BlogPost() -> Element {
    let post = use_loader_data::<BlogPost>();
    
    <article>
        <h1>{post.title}</h1>
        <div>{post.content}</div>
    </article>
}
```

### 3.2 Parallel Loading

Loading multiple data sources in parallel.

```rsx
async fn loader(params: RouteParams) -> Result<(User, Posts)> {
    let user_id = params.get("id")?;
    
    let (user, posts) = tokio::join!(
        fetch_user(user_id),
        fetch_user_posts(user_id),
    );
    
    Ok((user?, posts?))
}
```

### 3.3 Error Handling

Handling loader errors.

```rsx
async fn loader(params: RouteParams) -> Result<Data> {
    fetch_data().await
        .map_err(|e| RouteError::NotFound(e.to_string()))
}

fn Component() -> Element {
    match use_loader_data::<Data>() {
        Ok(data) => <DataView data={data} />,
        Err(e) => <ErrorBoundary error={e} />,
    }
}
```

## 4. Action Functions

### 4.1 Form Actions

Handling form submissions.

```rsx
// routes/users/[id]/edit.rsx
async fn action(request: Request) -> Result<Redirect> {
    let form_data = request.form_data()?;
    let user_id = request.params().get("id")?;
    
    update_user(user_id, form_data).await?;
    
    Ok(Redirect::to(format!("/users/{}", user_id)))
}

fn EditUser() -> Element {
    <form method="post">
        <input name="name" />
        <button type="submit">Save</button>
    </form>
}
```

### 4.2 Mutation Actions

Handling data mutations.

```rsx
async fn action(request: Request) -> Result<Json<Response>> {
    let data = request.json::<CreateData>()?;
    let result = create_resource(data).await?;
    Ok(Json(result))
}
```

## 5. Universal Routing

### 5.1 Server-Side Rendering

Rendering routes on the server.

```rsx
// Server-side
async fn handle_request(request: Request) -> Response {
    let route = match_route(&request.path)?;
    let data = route.loader().await?;
    let html = render_to_string(route.component(data));
    Response::html(html)
}
```

### 5.2 Client-Side Routing

Client-side navigation.

```rsx
fn Navigation() -> Element {
    let navigate = use_navigate();
    
    <nav>
        <Link to="/">Home</Link>
        <Link to="/about">About</Link>
        <button on_click={|| navigate("/contact")}>
            Contact
        </button>
    </nav>
}
```

### 5.3 Hybrid Routing

Combining server and client routing.

```rsx
// Initial load: server-rendered
// Navigation: client-side
// Data fetching: server API
```

## 6. Route Guards and Middleware

### 6.1 Route Guards

Protecting routes.

```rsx
// routes/dashboard/layout.rsx
fn DashboardLayout(children: Element) -> Element {
    let user = use_auth();
    
    if !user.is_authenticated() {
        return <Redirect to="/login" />;
    }
    
    <div>
        {children}
    </div>
}
```

### 6.2 Middleware

Route middleware.

```rsx
async fn middleware(request: Request) -> Result<Request> {
    // Log request
    log_request(&request);
    
    // Add headers
    request.headers_mut().insert("X-Custom", "value");
    
    Ok(request)
}
```

### 6.3 Route Meta

Route metadata.

```rsx
#[route_meta]
struct RouteMeta {
    title: String,
    description: String,
    requires_auth: bool,
    roles: Vec<Role>,
}

fn Component() -> Element {
    // Route metadata available
}
```

## 7. Navigation

### 7.1 Link Component

Declarative navigation.

```rsx
<Link to="/about">About</Link>
<Link to="/blog/[slug]" params={{"slug": "my-post"}}>
    My Post
</Link>
```

### 7.2 Programmatic Navigation

Navigating programmatically.

```rsx
fn Component() -> Element {
    let navigate = use_navigate();
    
    <button on_click={|| navigate("/new-page")}>
        Go
    </button>
}
```

### 7.3 Navigation State

Accessing navigation state.

```rsx
fn Component() -> Element {
    let location = use_location();
    let params = use_params();
    let query = use_query();
    
    <div>
        <p>Path: {location.pathname}</p>
        <p>Params: {params}</p>
        <p>Query: {query}</p>
    </div>
}
```

## 8. Route Matching

### 8.1 Route Matching Algorithm

Matching routes to URLs.

```rust
struct RouteMatcher {
    routes: Vec<Route>,
}

impl RouteMatcher {
    fn match_route(&self, path: &str) -> Option<&Route> {
        for route in &self.routes {
            if route.matches(path) {
                return Some(route);
            }
        }
        None
    }
}
```

### 8.2 Dynamic Segments

Matching dynamic segments.

```rust
struct Route {
    pattern: RoutePattern,
}

enum RoutePattern {
    Static(String),
    Dynamic(String), // [id]
    CatchAll(String), // [...rest]
}
```

### 8.3 Route Priority

Route matching priority.

```rust
// More specific routes matched first
// Static routes before dynamic
// Specific dynamic before catch-all
```

## 9. Route Configuration

### 9.1 Route Config

Configuring routes.

```toml
# rux.toml
[routing]
base_path = "/app"
trailing_slash = false
case_sensitive = true
```

### 9.2 Custom Routes

Defining custom routes.

```rsx
// routes.config.rsx
routes! {
    "/custom" => CustomComponent,
    "/api/*" => ApiHandler,
}
```

## 10. Future Considerations

- Route transitions
- Route prefetching
- Route code splitting
- Route analytics
- Route testing utilities

