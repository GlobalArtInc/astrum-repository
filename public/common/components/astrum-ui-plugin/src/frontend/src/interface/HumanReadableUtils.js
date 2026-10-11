

import UIStrings from '../constants/UIStrings';
import fileSize from 'file-size';
import { pageLanguage } from './plural';

/**
 * @since 3.41
 *
 * Utility methods for creating human readable strings from data.
 */
class HumanReadableUtils {
  /**
   * Convert a size in bytes to a human readable string
   * @param bytes
   * @param unitOfMeasurement - si (default), iec, or jedec, more information available at https://www.npmjs.com/package/file-size
   * @return {*}
   */
  static bytesToString(bytes, unitNotation = 'si') {
    if (bytes < 0) {
      return UIStrings.UNAVAILABLE;
    }
    const size = fileSize(bytes).calculate(unitNotation);
    const language = pageLanguage();
    const number = language.startsWith('en')
      ? size.fixed
      : new Intl.NumberFormat(language, {minimumFractionDigits: 2, maximumFractionDigits: 2, useGrouping: false})
          .format(Number(size.fixed));
    const unit = UIStrings.FILE_SIZE?.[size.suffix];
    return unit ? unit(number) : `${number} ${size.suffix}`;
  }
}

export default HumanReadableUtils;

// Named re-export so consumers using `import { HumanReadableUtils } from '...'` get
// the same class as the default export under Babel CJS interop.
export { HumanReadableUtils };
