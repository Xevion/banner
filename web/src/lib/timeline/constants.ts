/** Layout & padding */
export const PADDING = { top: 20, right: 0, bottom: 40, left: 0 } as const;
export const DEFAULT_AXIS_RATIO = 0.8;
export const CHART_HEIGHT_RATIO = 0.6;

/** Viewport span limits (ms) */
export const MIN_SPAN_MS = 5 * 60 * 1000;
export const MAX_SPAN_MS = 72 * 60 * 60 * 1000;
export const DEFAULT_SPAN_MS = 36 * 60 * 60 * 1000;

/** Slot interval (15 minutes) */
export const SLOT_INTERVAL_MS = 15 * 60 * 1000;

/** Zoom */
export const ZOOM_FACTOR = 1.15;
export const ZOOM_KEY_FACTOR = 1.25;
export const ZOOM_EASE = 0.12;
export const ZOOM_SETTLE_THRESHOLD = 100;

/** Grid rendering */
export const GRID_ALPHA = 0.15;
export const HOUR_GRID_ALPHA = 0.25;

/** Now line */
export const NOW_LINE_WIDTH = 2;
export const NOW_LINE_COLOR = "#ef4444";
export const NOW_TRIANGLE_HEIGHT = 6;
export const NOW_TRIANGLE_HALF_WIDTH = 5;
export const NOW_LABEL_FONT = "bold 10px Inter, system-ui, sans-serif";

/** Hover highlight */
export const HOVER_HIGHLIGHT_ALPHA = 0.25;

/** Stacked area */
export const AREA_FILL_ALPHA = 0.82;
export const AREA_STROKE_ALPHA = 0.4;

/** Physics / momentum */
export const PAN_FRICTION = 0.94;
export const PAN_STOP_THRESHOLD = 0.5;
export const PAN_STOP_THRESHOLD_Y = 0.01;
export const VELOCITY_SAMPLE_WINDOW = 80;
export const VELOCITY_MIN_DT = 5;

/** Keyboard panning */
export const PAN_STEP_RATIO = 0.1;
export const PAN_STEP_CTRL_RATIO = 0.35;
export const PAN_EASE = 0.12;
export const PAN_SETTLE_THRESHOLD_PX = 1;

/** Y-axis panning */
export const YRATIO_STEP = 0.03;
export const YRATIO_MIN = 0.3;
export const YRATIO_MAX = 1.2;
export const YRATIO_SETTLE_THRESHOLD = 0.001;

/** Follow mode */
export const FOLLOW_EASE = 0.03;

/** Animation */
export const VALUE_EASE = 0.12;
export const MAXY_EASE = 0.4;
export const SETTLE_THRESHOLD = 0.5;
export const MIN_MAXY = 60;
export const MAX_DT = 50;
export const DEFAULT_DT = 16;

/** Visible-slot margin for smooth curve edges at viewport boundaries */
export const RENDER_MARGIN_SLOTS = 3;

/** Drawer */
export const DRAWER_WIDTH = 220;

/** Touch / tap */
export const TAP_MAX_DURATION_MS = 250;
export const TAP_MAX_DISTANCE_PX = 10;

/** Axis text */
export const AXIS_FONT = "11px Inter, system-ui, sans-serif";
