type ScreenshotCrop = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type ScreenshotDimensions = { width: number; height: number };

/** Maps CSS pixels to image-relative snapshot pixels, preserving edge pixels. */
export function normalizeScreenshotCrop(
  crop: ScreenshotCrop,
  viewport: ScreenshotDimensions,
  image: ScreenshotDimensions,
  minSize = 8,
): ScreenshotCrop | null {
  if (
    ![
      crop.x,
      crop.y,
      crop.width,
      crop.height,
      viewport.width,
      viewport.height,
      image.width,
      image.height,
      minSize,
    ].every(Number.isFinite) ||
    crop.width <= 0 ||
    crop.height <= 0 ||
    viewport.width <= 0 ||
    viewport.height <= 0 ||
    image.width <= 0 ||
    image.height <= 0 ||
    !Number.isSafeInteger(image.width) ||
    !Number.isSafeInteger(image.height) ||
    minSize <= 0 ||
    !Number.isInteger(minSize)
  ) {
    return null;
  }

  const scaleX = image.width / viewport.width;
  const scaleY = image.height / viewport.height;
  const right = crop.x + crop.width;
  const bottom = crop.y + crop.height;
  const leftPixel = crop.x * scaleX;
  const topPixel = crop.y * scaleY;
  const rightPixel = right * scaleX;
  const bottomPixel = bottom * scaleY;
  if (
    ![
      scaleX,
      scaleY,
      right,
      bottom,
      leftPixel,
      topPixel,
      rightPixel,
      bottomPixel,
    ].every(Number.isFinite) ||
    scaleX <= 0 ||
    scaleY <= 0 ||
    right <= crop.x ||
    bottom <= crop.y ||
    crop.x >= viewport.width ||
    crop.y >= viewport.height ||
    right <= 0 ||
    bottom <= 0
  ) {
    return null;
  }

  const x = Math.min(image.width, Math.max(0, Math.floor(leftPixel)));
  const y = Math.min(image.height, Math.max(0, Math.floor(topPixel)));
  const endX = Math.min(image.width, Math.max(0, Math.ceil(rightPixel)));
  const endY = Math.min(image.height, Math.max(0, Math.ceil(bottomPixel)));
  const width = endX - x;
  const height = endY - y;

  return width >= minSize && height >= minSize
    ? { x, y, width, height }
    : null;
}
