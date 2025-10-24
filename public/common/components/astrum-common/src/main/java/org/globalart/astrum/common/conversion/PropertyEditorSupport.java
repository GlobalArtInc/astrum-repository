
package org.globalart.astrum.common.conversion;

import java.awt.Component;
import java.awt.Graphics;
import java.awt.Rectangle;
import java.beans.PropertyChangeListener;
import java.beans.PropertyEditor;

public abstract class PropertyEditorSupport
    implements PropertyEditor
{
  protected Object value;

  @Override
  public void setAsText(final String text) throws IllegalArgumentException {
    setValue(text);
  }

  @Override
  public Object getValue() {
    return value;
  }

  @Override
  public boolean isPaintable() {
    return false;
  }

  @Override
  public void paintValue(final Graphics gfx, final Rectangle box) {
    throw new UnsupportedOperationException();
  }

  @Override
  public String getJavaInitializationString() {
    throw new UnsupportedOperationException();
  }

  @Override
  public String getAsText() {
    return value != null ? value.toString() : null;
  }

  @Override
  public String[] getTags() {
    return null;
  }

  @Override
  public Component getCustomEditor() {
    throw new UnsupportedOperationException();
  }

  @Override
  public boolean supportsCustomEditor() {
    return false;
  }

  @Override
  public void addPropertyChangeListener(final PropertyChangeListener listener) {
    throw new UnsupportedOperationException();
  }

  @Override
  public void removePropertyChangeListener(final PropertyChangeListener listener) {
    throw new UnsupportedOperationException();
  }
}
