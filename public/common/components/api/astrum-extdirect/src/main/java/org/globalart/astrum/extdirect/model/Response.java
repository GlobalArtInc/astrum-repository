
package org.globalart.astrum.extdirect.model;

/**
 * Ext.Direct response.
 *
 * @since 3.0
 */
public class Response<T>
{
  private boolean success;

  private T data;

  public Response(boolean success, T data) {
    this.success = success;
    this.data = data;
  }

  public boolean isSuccess() {
    return success;
  }

  public T getData() {
    return data;
  }
}
