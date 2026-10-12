package org.globalart.astrum.repository.rubygems.internal.marshal;

import java.io.IOException;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.rubygems.internal.gem.GemRequirements;

public class RubyMarshalWriter
{
  public static final int MAJOR = 4;

  public static final int MINOR = 8;

  private static final int FIXNUM_MAX = (1 << 30) - 1;

  private static final int FIXNUM_MIN = -(1 << 30);

  private final OutputStream out;

  private final Map<String, Integer> symbols = new HashMap<>();

  public RubyMarshalWriter(final OutputStream out) {
    this(out, true);
  }

  public RubyMarshalWriter(final OutputStream out, final boolean header) {
    this.out = out;
    if (header) {
      write(MAJOR);
      write(MINOR);
    }
  }

  public RubyMarshalWriter nil() {
    write('0');
    return this;
  }

  public RubyMarshalWriter bool(final boolean value) {
    write(value ? 'T' : 'F');
    return this;
  }

  public RubyMarshalWriter integer(final long value) {
    if (value < FIXNUM_MIN || value > FIXNUM_MAX) {
      throw new IllegalArgumentException("Integer out of range: " + value);
    }
    write('i');
    writeLong(value);
    return this;
  }

  public RubyMarshalWriter string(final String value) {
    if (value == null) {
      return nil();
    }
    write('I');
    write('"');
    writeBytes(value.getBytes(StandardCharsets.UTF_8));
    writeLong(1);
    symbol("E");
    write('T');
    return this;
  }

  public RubyMarshalWriter binary(final byte[] value) {
    write('"');
    writeBytes(value);
    return this;
  }

  public RubyMarshalWriter symbol(final String name) {
    Integer index = symbols.get(name);
    if (index != null) {
      write(';');
      writeLong(index);
    }
    else {
      symbols.put(name, symbols.size());
      write(':');
      writeBytes(name.getBytes(StandardCharsets.UTF_8));
    }
    return this;
  }

  public RubyMarshalWriter array(final int size) {
    write('[');
    writeLong(size);
    return this;
  }

  public RubyMarshalWriter hash(final int size) {
    write('{');
    writeLong(size);
    return this;
  }

  public RubyMarshalWriter strings(final List<String> values) {
    array(values.size());
    values.forEach(this::string);
    return this;
  }

  public RubyMarshalWriter userMarshal(final String className) {
    write('U');
    symbol(className);
    return this;
  }

  public RubyMarshalWriter userDefined(final String className, final byte[] data) {
    write('u');
    symbol(className);
    writeBytes(data);
    return this;
  }

  public RubyMarshalWriter object(final String className, final int instanceVariables) {
    write('o');
    symbol(className);
    writeLong(instanceVariables);
    return this;
  }

  public RubyMarshalWriter gemVersion(final String version) {
    userMarshal("Gem::Version");
    array(1);
    string(version);
    return this;
  }

  public RubyMarshalWriter gemRequirement(final List<String> requirements) {
    List<String> effective = requirements.isEmpty() ? List.of(GemRequirements.ANY) : requirements;
    userMarshal("Gem::Requirement");
    array(1);
    array(effective.size());
    for (String requirement : effective) {
      String[] parts = GemRequirements.split(requirement);
      array(2);
      string(parts[0]);
      gemVersion(parts[1]);
    }
    return this;
  }

  void writeLong(final long value) {
    if (value == 0) {
      write(0);
      return;
    }
    if (value > 0 && value < 123) {
      write((int) value + 5);
      return;
    }
    if (value > -124 && value < 0) {
      write((int) ((value - 5) & 0xff));
      return;
    }
    byte[] buffer = new byte[9];
    long x = value;
    int i;
    for (i = 1; i < 9; i++) {
      buffer[i] = (byte) (x & 0xff);
      x >>= 8;
      if (x == 0) {
        buffer[0] = (byte) i;
        break;
      }
      if (x == -1) {
        buffer[0] = (byte) -i;
        break;
      }
    }
    write(buffer, i + 1);
  }

  private void writeBytes(final byte[] bytes) {
    writeLong(bytes.length);
    write(bytes, bytes.length);
  }

  private void write(final int b) {
    try {
      out.write(b);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private void write(final byte[] bytes, final int length) {
    try {
      out.write(bytes, 0, length);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
