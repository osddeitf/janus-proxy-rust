
/** Unauthorized (can only happen when using apisecret/auth token) */
pub static JANUS_ERROR_UNAUTHORIZED: u32 = 403;

/** Unauthorized access to a plugin (can only happen when using auth token) */
pub static JANUS_ERROR_UNAUTHORIZED_PLUGIN: u32 = 405;

/** Unknown/undocumented error */
pub static JANUS_ERROR_UNKNOWN: u32 = 490;

/** Transport related error */
pub static JANUS_ERROR_TRANSPORT_SPECIFIC: u32 = 450;

/** The request is missing in the message */
pub static JANUS_ERROR_MISSING_REQUEST: u32 = 452;

/** The Janus core does not support this request */
pub static JANUS_ERROR_UNKNOWN_REQUEST: u32 = 453;

/** The payload is not a valid JSON message */
pub static JANUS_ERROR_INVALID_JSON: u32 = 454;

/** The object is not a valid JSON object as expected */
pub static JANUS_ERROR_INVALID_JSON_OBJECT: u32 = 455;

/** A mandatory element is missing in the message */
pub static JANUS_ERROR_MISSING_MANDATORY_ELEMENT: u32 = 456;

/** The request cannot be handled for this webserver path  */
pub static JANUS_ERROR_INVALID_REQUEST_PATH: u32 = 457;

/** The session the request refers to doesn't exist */
pub static JANUS_ERROR_SESSION_NOT_FOUND: u32 = 458;

/** The handle the request refers to doesn't exist */
pub static JANUS_ERROR_HANDLE_NOT_FOUND: u32 = 459;

/** The plugin the request wants to talk to doesn't exist */
pub static JANUS_ERROR_PLUGIN_NOT_FOUND: u32 = 460;

/** An error occurring when trying to attach to a plugin and create a handle  */
pub static JANUS_ERROR_PLUGIN_ATTACH: u32 = 461;

/** An error occurring when trying to send a message/request to the plugin */
pub static JANUS_ERROR_PLUGIN_MESSAGE: u32 = 462;

/** An error occurring when trying to detach from a plugin and destroy the related handle  */
pub static JANUS_ERROR_PLUGIN_DETACH: u32 = 463;

/** The Janus core doesn't support this SDP type */
pub static JANUS_ERROR_JSEP_UNKNOWN_TYPE: u32 = 464;

/** The Session Description provided by the peer is invalid */
pub static JANUS_ERROR_JSEP_INVALID_SDP: u32 = 465;

/** The stream a trickle candidate for does not exist or is invalid */
pub static JANUS_ERROR_TRICKE_INVALID_STREAM: u32 = 466;

/** A JSON element is of the wrong type (e.g., an integer instead of a string) */
pub static JANUS_ERROR_INVALID_ELEMENT_TYPE: u32 = 467;

/** The ID provided to create a new session is already in use */
pub static JANUS_ERROR_SESSION_CONFLICT: u32 = 468;

/** We got an ANSWER to an OFFER we never made */
pub static JANUS_ERROR_UNEXPECTED_ANSWER: u32 = 469;

/** The auth token the request refers to doesn't exist */
pub static JANUS_ERROR_TOKEN_NOT_FOUND: u32 = 470;

/** The current request cannot be handled because of not compatible WebRTC state */
pub static JANUS_ERROR_WEBRTC_STATE: u32 = 471;

/** The server is currently configured not to accept new sessions */
pub static JANUS_ERROR_NOT_ACCEPTING_SESSIONS: u32 = 472;

/** Proxy error: request to janus-gateway timeout */
pub static JANUS_ERROR_GATEWAY_TIMED_OUT: u32 = 500;

/** Proxy error: programmer error, or unexpected (should never occurred) */
pub static JANUS_ERROR_GATEWAY_INTERNAL_ERROR: u32 = 599;

/** Proxy error: connect websocket to janus-gateway failed */
pub static JANUS_ERROR_GATEWAY_CONNECTION_FAILED: u32 = 501;

/** Proxy error: No janus-gateway instance available */
pub static JANUS_ERROR_GATEWAY_UNAVAILABLE: u32 = 502;

/** Proxy error: janus-gateway connection closed */
pub static JANUS_ERROR_GATEWAY_CONNECTION_CLOSED: u32 = 503;
