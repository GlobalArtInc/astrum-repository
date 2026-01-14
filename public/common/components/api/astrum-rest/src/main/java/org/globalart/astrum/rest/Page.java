
package org.globalart.astrum.rest;

import java.util.Objects;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * An object to hold pages of XOs for use in the REST API.
 *
 * @since 3.3
 */
public class Page<T>
{

  private List<T> items;

  private String continuationToken;

  public Page(
      @JsonProperty("items") List<T> items,
      @JsonProperty("continuationToken") String continuationToken)
  {
    this.items = items;
    this.continuationToken = continuationToken;
  }

  public List<T> getItems() {
    return items;
  }

  public void setItems(List<T> items) {
    this.items = items;
  }

  public String getContinuationToken() {
    return continuationToken;
  }

  public void setContinuationToken(String continuationToken) {
    this.continuationToken = continuationToken;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o)
      return true;
    if (o == null || getClass() != o.getClass())
      return false;
    Page<?> page = (Page<?>) o;
    return Objects.equals(items, page.items) &&
        Objects.equals(continuationToken, page.continuationToken);
  }

  @Override
  public int hashCode() {
    return Objects.hash(items, continuationToken);
  }

  @Override
  public String toString() {
    return "Page{" +
        "items=" + items +
        ", continuationToken='" + continuationToken + '\'' +
        '}';
  }
}
