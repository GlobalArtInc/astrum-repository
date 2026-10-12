package org.globalart.astrum.repository.rubygems;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsFormat
    extends Format
{
  public static final String NAME = "rubygems";

  public RubygemsFormat() {
    super(NAME);
  }
}
