
package org.globalart.astrum.repository.upload;

import java.util.Arrays;
import java.util.List;

/**
 *
 * @since 3.8
 */
public class UploadRegexMap
{
  private final String regex;

  private final List<String> fieldList;

  public UploadRegexMap(final String regex, final String... fieldList) {
    this.regex = regex;
    this.fieldList = Arrays.asList(fieldList);
  }

  /**
   * Regex to be used to match against the file input. Groups from the regex will be mapped to the field list.
   */
  public String getRegex() {
    return regex;
  }

  /**
   * List of fields to populate from regex groups. (Use <code>null</code> for unused regex groups)
   */
  public List<String> getFieldList() {
    return fieldList;
  }
}
