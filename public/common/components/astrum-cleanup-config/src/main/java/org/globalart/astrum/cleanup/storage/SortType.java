
package org.globalart.astrum.cleanup.storage;

public enum SortType
{
  VERSION("version"),
  DATE("date");

  public final String value;

  SortType(final String value) {
    this.value = value;
  }
}
