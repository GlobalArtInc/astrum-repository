package org.globalart.astrum.repository.yum;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(YumFormat.NAME)
public class YumFormat
    extends Format
{
  public static final String NAME = "yum";

  public YumFormat() {
    super(NAME);
  }
}
