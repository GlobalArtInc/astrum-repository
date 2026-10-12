package org.globalart.astrum.repository.yum.internal.rpm;

import javax.annotation.Nullable;

public record RpmDependency(
    String name,
    @Nullable String flags,
    @Nullable String epoch,
    @Nullable String version,
    @Nullable String release,
    boolean pre)
{
  static final int SENSE_LESS = 1 << 1;

  static final int SENSE_GREATER = 1 << 2;

  static final int SENSE_EQUAL = 1 << 3;

  static final int SENSE_POSTTRANS = 1 << 5;

  static final int SENSE_PREREQ = 1 << 6;

  static final int SENSE_PRETRANS = 1 << 7;

  static final int SENSE_SCRIPT_PRE = 1 << 9;

  static final int SENSE_SCRIPT_POST = 1 << 10;

  static final int SENSE_RPMLIB = 1 << 24;

  static final int PRE_MASK = SENSE_PREREQ | SENSE_PRETRANS | SENSE_POSTTRANS | SENSE_SCRIPT_PRE | SENSE_SCRIPT_POST;

  public static RpmDependency of(final String name, final long senseFlags, @Nullable final String evr, final boolean pre) {
    String flags = flags(senseFlags);
    if (flags == null || evr == null || evr.isEmpty()) {
      return new RpmDependency(name, null, null, null, null, pre);
    }
    Evr parsed = Evr.parse(evr);
    return new RpmDependency(name, flags, parsed.epoch(), parsed.version(), parsed.release(), pre);
  }

  @Nullable
  static String flags(final long senseFlags) {
    long comparison = senseFlags & (SENSE_LESS | SENSE_GREATER | SENSE_EQUAL);
    if (comparison == SENSE_LESS) {
      return "LT";
    }
    if (comparison == SENSE_GREATER) {
      return "GT";
    }
    if (comparison == SENSE_EQUAL) {
      return "EQ";
    }
    if (comparison == (SENSE_LESS | SENSE_EQUAL)) {
      return "LE";
    }
    if (comparison == (SENSE_GREATER | SENSE_EQUAL)) {
      return "GE";
    }
    return null;
  }

  public record Evr(String epoch, String version, @Nullable String release)
  {
    public static Evr parse(final String evr) {
      String epoch = "0";
      String rest = evr;
      int colon = evr.indexOf(':');
      if (colon > 0 && evr.substring(0, colon).chars().allMatch(Character::isDigit)) {
        epoch = evr.substring(0, colon);
        rest = evr.substring(colon + 1);
      }
      int dash = rest.lastIndexOf('-');
      if (dash < 0) {
        return new Evr(epoch, rest, null);
      }
      return new Evr(epoch, rest.substring(0, dash), rest.substring(dash + 1));
    }
  }
}
