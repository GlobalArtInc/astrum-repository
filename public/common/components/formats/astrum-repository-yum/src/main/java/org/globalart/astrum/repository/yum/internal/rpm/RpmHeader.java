package org.globalart.astrum.repository.yum.internal.rpm;

import java.io.DataInputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public final class RpmHeader
{
  public static final int MAGIC = 0x8EADE801;

  public static final int TYPE_NULL = 0;

  public static final int TYPE_CHAR = 1;

  public static final int TYPE_INT8 = 2;

  public static final int TYPE_INT16 = 3;

  public static final int TYPE_INT32 = 4;

  public static final int TYPE_INT64 = 5;

  public static final int TYPE_STRING = 6;

  public static final int TYPE_BIN = 7;

  public static final int TYPE_STRING_ARRAY = 8;

  public static final int TYPE_I18NSTRING = 9;

  static final int MAX_ENTRIES = 0xffff;

  static final int MAX_DATA = 64 * 1024 * 1024;

  private final Map<Integer, Entry> entries;

  private final byte[] data;

  private final int entryCount;

  private RpmHeader(final Map<Integer, Entry> entries, final byte[] data, final int entryCount) {
    this.entries = entries;
    this.data = data;
    this.entryCount = entryCount;
  }

  public static RpmHeader read(final DataInputStream in) throws IOException {
    int magic = in.readInt();
    if (magic != MAGIC) {
      throw new IllegalArgumentException("Bad RPM header magic " + Integer.toHexString(magic));
    }
    in.readInt();
    int count = in.readInt();
    int length = in.readInt();
    if (count < 0 || count > MAX_ENTRIES) {
      throw new IllegalArgumentException("RPM header has an invalid number of entries: " + Integer.toUnsignedString(count));
    }
    if (length < 0 || length > MAX_DATA) {
      throw new IllegalArgumentException("RPM header has an invalid data size: " + Integer.toUnsignedString(length));
    }
    byte[] index = new byte[count * 16];
    in.readFully(index);
    byte[] data = new byte[length];
    in.readFully(data);
    Map<Integer, Entry> entries = new HashMap<>();
    for (int i = 0; i < count; i++) {
      int base = i * 16;
      Entry entry = new Entry(intAt(index, base + 4), intAt(index, base + 8), intAt(index, base + 12));
      if (entry.offset < 0 || entry.offset > length || entry.count < 0) {
        throw new IllegalArgumentException("RPM header entry " + intAt(index, base) + " points outside the header");
      }
      entries.putIfAbsent(intAt(index, base), entry);
    }
    return new RpmHeader(entries, data, count);
  }

  public long size() {
    return 16L + 16L * entryCount + data.length;
  }

  public boolean has(final int tag) {
    return entries.containsKey(tag);
  }

  public Optional<String> string(final int tag) {
    List<String> values = strings(tag);
    return values.isEmpty() ? Optional.empty() : Optional.of(values.get(0));
  }

  public List<String> strings(final int tag) {
    Entry entry = entries.get(tag);
    if (entry == null) {
      return List.of();
    }
    int count = switch (entry.type) {
      case TYPE_STRING -> 1;
      case TYPE_STRING_ARRAY, TYPE_I18NSTRING -> entry.count;
      default -> 0;
    };
    List<String> values = new ArrayList<>(Math.min(count, 4096));
    int position = entry.offset;
    for (int i = 0; i < count; i++) {
      int end = position;
      while (end < data.length && data[end] != 0) {
        end++;
      }
      if (end >= data.length) {
        throw new IllegalArgumentException("Unterminated string in RPM header entry " + tag);
      }
      values.add(new String(data, position, end - position, StandardCharsets.UTF_8));
      position = end + 1;
    }
    return values;
  }

  public long[] numbers(final int tag) {
    Entry entry = entries.get(tag);
    if (entry == null) {
      return new long[0];
    }
    int width = switch (entry.type) {
      case TYPE_CHAR, TYPE_INT8 -> 1;
      case TYPE_INT16 -> 2;
      case TYPE_INT32 -> 4;
      case TYPE_INT64 -> 8;
      default -> 0;
    };
    if (width == 0) {
      return new long[0];
    }
    if ((long) entry.offset + (long) entry.count * width > data.length) {
      throw new IllegalArgumentException("RPM header entry " + tag + " points outside the header");
    }
    long[] values = new long[entry.count];
    for (int i = 0; i < entry.count; i++) {
      int position = entry.offset + i * width;
      values[i] = switch (width) {
        case 1 -> data[position] & 0xffL;
        case 2 -> ((data[position] & 0xff) << 8) | (data[position + 1] & 0xff);
        case 4 -> intAt(data, position) & 0xffffffffL;
        default -> ((long) intAt(data, position) << 32) | (intAt(data, position + 4) & 0xffffffffL);
      };
    }
    return values;
  }

  public Optional<Long> number(final int tag) {
    long[] values = numbers(tag);
    return values.length == 0 ? Optional.empty() : Optional.of(values[0]);
  }

  private static int intAt(final byte[] bytes, final int position) {
    return ((bytes[position] & 0xff) << 24) | ((bytes[position + 1] & 0xff) << 16)
        | ((bytes[position + 2] & 0xff) << 8) | (bytes[position + 3] & 0xff);
  }

  private record Entry(int type, int offset, int count)
  {
  }
}
