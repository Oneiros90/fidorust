export const RIGHT_PANEL_COLLAPSE_QUERY = '(max-width: 768px)';
export const DRAG_THRESHOLD_PX = 5;
export const THUMB_SIZE = 40;
export const PREVIEW_ROOT_MARGIN = '160px';
export const RECENTS_MAX = 10;
export const PDF_MAX_PT = 720;
/** One FidoCAD logical unit is 127 µm, i.e. 200 LU per inch. */
export const MM_PER_LU = 0.127;
export const LU_PER_INCH = 200;
export const PT_PER_LU = (MM_PER_LU / 25.4) * 72;
export const PNG_PPI_PRESETS = [72, 96, 150, 200, 300, 600, 1200] as const;
export const PNG_MAX_EDGE = 8192;
export const A4_PT = { w: 595.28, h: 841.89 };
export const LETTER_PT = { w: 612, h: 792 };
