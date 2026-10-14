import { describe, expect, it } from "vitest";
import { normalizeScreenshotCrop } from "./screenshotCoordinates";

const viewport = { width: 100, height: 80 };
const image = { width: 100, height: 80 };
const crop = { x: 10, y: 20, width: 30, height: 40 };

describe("normalizeScreenshotCrop", () => {
  it("maps a Retina 2x viewport into image-relative snapshot pixels", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 10, y: 20, width: 30, height: 40 },
        { width: 1440, height: 900 },
        { width: 2880, height: 1800 },
      ),
    ).toEqual({ x: 20, y: 40, width: 60, height: 80 });
  });

  it("rounds outward at 150% scaling with fractional CSS coordinates", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 10.2, y: 20.4, width: 30.1, height: 40.2 },
        { width: 800, height: 600 },
        { width: 1200, height: 900 },
      ),
    ).toEqual({ x: 15, y: 30, width: 46, height: 61 });
  });

  it("uses independent horizontal and vertical scale factors", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 10, y: 20, width: 30, height: 40 },
        { width: 100, height: 100 },
        { width: 200, height: 150 },
      ),
    ).toEqual({ x: 20, y: 30, width: 60, height: 60 });
  });

  it("preserves both edge pixels instead of rounding origin and size separately", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 10.75, y: 20.75, width: 7.5, height: 7.5 },
        viewport,
        image,
      ),
    ).toEqual({ x: 10, y: 20, width: 9, height: 9 });
  });

  it("clamps negative origins to the image without moving the far edge", () => {
    expect(
      normalizeScreenshotCrop(
        { x: -5.5, y: -3.5, width: 20, height: 20 },
        viewport,
        image,
      ),
    ).toEqual({ x: 0, y: 0, width: 15, height: 17 });
  });

  it("clamps right and bottom edges to the image", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 90.2, y: 70.2, width: 30, height: 30 },
        viewport,
        image,
      ),
    ).toEqual({ x: 90, y: 70, width: 10, height: 10 });
  });

  it("clamps a selection extending beyond every image edge", () => {
    expect(
      normalizeScreenshotCrop(
        { x: -10, y: -10, width: 130, height: 110 },
        viewport,
        image,
      ),
    ).toEqual({ x: 0, y: 0, width: 100, height: 80 });
  });

  it.each([
    { x: -20, y: 20, width: 10, height: 20 },
    { x: 100, y: 20, width: 10, height: 20 },
    { x: 10, y: -20, width: 20, height: 10 },
    { x: 10, y: 80, width: 20, height: 10 },
    { x: -10, y: 20, width: 10, height: 20 },
    { x: 10, y: -10, width: 20, height: 10 },
  ])("rejects a selection completely outside or touching the image: %o", (outside) => {
    expect(normalizeScreenshotCrop(outside, viewport, image)).toBeNull();
  });

  it.each(["x", "y"] as const)(
    "rejects a selection starting outside %s despite floating-point scale error",
    (axis) => {
      expect(
        normalizeScreenshotCrop(
          { x: 0, y: 0, width: 49, height: 49, [axis]: 49 },
          { width: 49, height: 49 },
          { width: 1, height: 1 },
          1,
        ),
      ).toBeNull();
    },
  );

  it("accepts the default minimum at exactly eight final image pixels", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 0, y: 0, width: 4, height: 4 },
        viewport,
        { width: 200, height: 160 },
      ),
    ).toEqual({ x: 0, y: 0, width: 8, height: 8 });
  });

  it.each([
    { x: 0, y: 0, width: 7, height: 8 },
    { x: 0, y: 0, width: 8, height: 7 },
    { x: 93, y: 0, width: 20, height: 20 },
    { x: 0, y: 73, width: 20, height: 20 },
    { x: -13, y: 0, width: 20, height: 20 },
    { x: 0, y: -13, width: 20, height: 20 },
  ])("applies the eight-pixel minimum after rounding and clamping: %o", (small) => {
    expect(normalizeScreenshotCrop(small, viewport, image)).toBeNull();
  });

  it("supports a custom positive integer minimum", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 0, y: 0, width: 1, height: 1 },
        viewport,
        image,
        1,
      ),
    ).toEqual({ x: 0, y: 0, width: 1, height: 1 });
    expect(normalizeScreenshotCrop(crop, viewport, image, 31)).toBeNull();
  });

  it.each([0, -1, 1.5, NaN, Infinity, -Infinity])(
    "rejects an invalid minimum size: %s",
    (minSize) => {
      expect(normalizeScreenshotCrop(crop, viewport, image, minSize)).toBeNull();
    },
  );

  it.each(["x", "y", "width", "height"] as const)(
    "rejects non-finite crop %s",
    (dimension) => {
      for (const value of [NaN, Infinity, -Infinity]) {
        expect(
          normalizeScreenshotCrop({ ...crop, [dimension]: value }, viewport, image),
        ).toBeNull();
      }
    },
  );

  it.each(["width", "height"] as const)(
    "rejects nonpositive crop %s",
    (dimension) => {
      for (const value of [0, -1]) {
        expect(
          normalizeScreenshotCrop({ ...crop, [dimension]: value }, viewport, image),
        ).toBeNull();
      }
    },
  );

  it.each(["width", "height"] as const)(
    "rejects non-finite or nonpositive viewport %s",
    (dimension) => {
      for (const value of [0, -1, NaN, Infinity, -Infinity]) {
        expect(
          normalizeScreenshotCrop(crop, { ...viewport, [dimension]: value }, image),
        ).toBeNull();
      }
    },
  );

  it.each(["width", "height"] as const)(
    "requires image %s to be a positive safe integer",
    (dimension) => {
      for (const value of [
        0,
        -1,
        1.5,
        NaN,
        Infinity,
        -Infinity,
        Number.MAX_SAFE_INTEGER + 1,
      ]) {
        expect(
          normalizeScreenshotCrop(crop, viewport, { ...image, [dimension]: value }),
        ).toBeNull();
      }
    },
  );

  it("supports fractional positive viewport dimensions", () => {
    expect(
      normalizeScreenshotCrop(
        { x: 0, y: 0, width: 12.5, height: 10.5 },
        { width: 12.5, height: 10.5 },
        { width: 25, height: 21 },
      ),
    ).toEqual({ x: 0, y: 0, width: 25, height: 21 });
  });

  it.each(["x", "y"] as const)(
    "rejects overflow when computing the CSS end on %s",
    (axis) => {
      const dimension = axis === "x" ? "width" : "height";
      expect(
        normalizeScreenshotCrop(
          { ...crop, [axis]: Number.MAX_VALUE, [dimension]: Number.MAX_VALUE },
          viewport,
          image,
        ),
      ).toBeNull();
    },
  );

  it.each(["width", "height"] as const)(
    "rejects scale overflow on %s before clamping",
    (dimension) => {
      expect(
        normalizeScreenshotCrop(crop, { ...viewport, [dimension]: Number.MIN_VALUE }, image),
      ).toBeNull();
    },
  );

  it.each(["x", "y"] as const)(
    "rejects a scaled endpoint overflow on %s before clamping",
    (axis) => {
      const dimension = axis === "x" ? "width" : "height";
      expect(
        normalizeScreenshotCrop(
          { ...crop, [axis]: 0, [dimension]: Number.MAX_VALUE },
          viewport,
          { width: 200, height: 160 },
        ),
      ).toBeNull();
    },
  );

  it("rejects a crop whose endpoint collapses due to numeric precision", () => {
    expect(
      normalizeScreenshotCrop(
        { x: Number.MAX_SAFE_INTEGER, y: 0, width: 0.1, height: 10 },
        viewport,
        image,
        1,
      ),
    ).toBeNull();
  });
});
