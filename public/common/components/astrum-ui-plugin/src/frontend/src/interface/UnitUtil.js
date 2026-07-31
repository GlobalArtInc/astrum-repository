

/**
 * @since 3.37
 */
export default class UnitUtil {
  static bytesToMegaBytes(value) {
    if (value) {
      return value / (1024 * 1024);
    }
    return value;
  }

  static megaBytesToBytes(value) {
    if (value) {
      return value * (1024 * 1024);
    }
    return value;
  }
}
