macro_rules! dynamic_path {
	($route:literal) => {{
		let mut route = warp::any().boxed();
		for segment in $route.trim_start_matches('/').split('/') {
			route = route.and(warp::path(segment)).boxed();
		}
		route
	}};
}

macro_rules! png_route {
	($path:literal) => {{
		const PNG_BYTES: &[u8] = include_bytes!(concat!("web_server/", $path));
		dynamic_path!($path)
			.map(|| warp::reply::with_header(PNG_BYTES, "Content-Type", "image/png"))
	}};
}

macro_rules! svg_route {
	($path:literal) => {{
		const SVG_STR: &str = include_str!(concat!("web_server/", $path));
		dynamic_path!($path)
			.map(|| warp::reply::with_header(SVG_STR, "Content-Type", "image/svg+xml"))
	}};
}

macro_rules! ico_route {
	($path:literal) => {{
		const ICO_BYTES: &[u8] = include_bytes!(concat!("web_server/", $path));
		dynamic_path!($path)
			.map(|| warp::reply::with_header(ICO_BYTES, "Content-Type", "image/x-icon"))
	}};
}

macro_rules! css_route {
	($path:literal) => {{
		const CSS_STR: &str = include_str!(concat!("web_server/", $path));
		dynamic_path!($path).map(|| warp::reply::with_header(CSS_STR, "Content-Type", "text/css"))
	}};
}

macro_rules! js_route {
	($path:literal) => {{
		const JS_STR: &str = include_str!(concat!("web_server/", $path));
		dynamic_path!($path)
			.map(|| warp::reply::with_header(JS_STR, "Content-Type", "application/javascript"))
	}};
}

macro_rules! json_route {
	($path:literal) => {{
		const JSON_STR: &str = include_str!(concat!("web_server/", $path));
		dynamic_path!($path)
			.map(|| warp::reply::with_header(JSON_STR, "Content-Type", "application/json"))
	}};
}

/// Only specify the directory in which the 'index.html' is located
/// 'index_html_route!("foo")' will accept '/foo' and '/foo/index.html' and will return the 'foo/index.html' file
macro_rules! index_html_route {
	($path:literal) => {{
		const HTML_STR: &str = include_str!(concat!("web_server/", concat!($path, "/index.html")));
		let base_route = dynamic_path!($path);
		base_route
			.clone()
			.and(warp::path::end())
			.or(base_route.and(warp::path("index.html")))
			.map(|_| warp::reply::html(HTML_STR))
	}};
}
