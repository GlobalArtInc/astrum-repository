
package org.globalart.astrum.freeze.event;

public abstract class FreezeEvent
{
  public enum FreezeEventTypes
  {
    FREEZE("Freeze"),
    RELEASE("Release"),
    FORCE_RELEASE("Force Release");

    private final String type;

    FreezeEventTypes(final String type) {
      this.type = type;
    }

    public String getType() {
      return this.type;
    }
  }

  private final FreezeEventTypes eventType;

  protected FreezeEvent(final FreezeEventTypes eventType) {
    this.eventType = eventType;
  }

  public FreezeEventTypes getEventType() {
    return eventType;
  }
}
