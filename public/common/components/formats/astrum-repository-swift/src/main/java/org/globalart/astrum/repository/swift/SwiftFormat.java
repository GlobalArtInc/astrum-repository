package org.globalart.astrum.repository.swift;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(SwiftFormat.NAME)
public class SwiftFormat
    extends Format
{
  public static final String NAME = "swift";

  public SwiftFormat() {
    super(NAME);
  }
}
