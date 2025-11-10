
package org.globalart.astrum.configuration.model;

import org.globalart.astrum.configuration.ApplyStatus;

public class ApplyStatusXO
{
  private ApplyStatus status;

  private String message;

  private String entity;

  public ApplyStatus getStatus() {
    return status;
  }

  public void setStatus(final ApplyStatus status) {
    this.status = status;
  }

  public String getMessage() {
    return message;
  }

  public void setMessage(final String message) {
    this.message = message;
  }

  public String getEntity() {
    return entity;
  }

  public void setEntity(final String entity) {
    this.entity = entity;
  }

  public static ApplyStatusXO from(final String entity, final String message, final ApplyStatus status) {
    ApplyStatusXO xo = new ApplyStatusXO();
    xo.setEntity(entity);
    xo.setMessage(message);
    xo.setStatus(status);
    return xo;
  }
}
