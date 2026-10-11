package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Collects component changes per channel until the channel has been quiet for a while, so that one deployment, which
 * stores a component and then several files of it, becomes one notice per component. A channel that keeps receiving
 * changes is flushed anyway once its oldest change has waited for the maximum delay.
 */
public class NotificationBuffer
{
  private static final int MAX_OVERFLOW_KEYS = 100_000;

  private final long quietMillis;

  private final long maxDelayMillis;

  private final int capacity;

  private final Map<String, Window> windows = new HashMap<>();

  public NotificationBuffer(final long quietMillis, final long maxDelayMillis, final int capacity) {
    checkArgument(quietMillis >= 0, "quietMillis must not be negative");
    checkArgument(maxDelayMillis >= quietMillis, "maxDelayMillis must not be shorter than quietMillis");
    checkArgument(capacity > 0, "capacity must be positive");
    this.quietMillis = quietMillis;
    this.maxDelayMillis = maxDelayMillis;
    this.capacity = capacity;
  }

  /**
   * The changes of one channel collected in one window, in the order the components first changed.
   *
   * @param notices the changes to tell about
   * @param overflow how many more components changed beyond the buffer's capacity
   */
  public record Batch(List<ComponentNotice> notices, int overflow)
  {
  }

  public synchronized void add(final String channelId, final ComponentNotice notice, final long now) {
    Window window = windows.computeIfAbsent(channelId, id -> new Window(now));
    window.lastAt = now;
    String key = notice.key();
    ComponentNotice pending = window.pending.get(key);
    if (pending != null) {
      ComponentNotice merged = pending.merge(notice);
      if (merged == null) {
        window.pending.remove(key);
      }
      else {
        window.pending.put(key, merged);
      }
    }
    else if (window.pending.size() < capacity) {
      window.pending.put(key, notice);
    }
    else if (window.overflow.size() < MAX_OVERFLOW_KEYS) {
      window.overflow.add(key);
    }
  }

  /**
   * Removes and returns the windows that have been quiet long enough or have waited for the maximum delay.
   */
  public synchronized Map<String, Batch> drainReady(final long now) {
    Map<String, Batch> ready = new LinkedHashMap<>();
    Iterator<Map.Entry<String, Window>> iterator = windows.entrySet().iterator();
    while (iterator.hasNext()) {
      Map.Entry<String, Window> entry = iterator.next();
      Window window = entry.getValue();
      if (now - window.lastAt >= quietMillis || now - window.firstAt >= maxDelayMillis) {
        iterator.remove();
        if (!window.isEmpty()) {
          ready.put(entry.getKey(), window.toBatch());
        }
      }
    }
    return ready;
  }

  /**
   * Removes and returns every window, ready or not.
   */
  public synchronized Map<String, Batch> drainAll() {
    Map<String, Batch> all = new LinkedHashMap<>();
    windows.forEach((channelId, window) -> {
      if (!window.isEmpty()) {
        all.put(channelId, window.toBatch());
      }
    });
    windows.clear();
    return all;
  }

  public synchronized boolean isEmpty() {
    return windows.isEmpty();
  }

  private static class Window
  {
    private final long firstAt;

    private long lastAt;

    private final LinkedHashMap<String, ComponentNotice> pending = new LinkedHashMap<>();

    private final Set<String> overflow = new HashSet<>();

    private Window(final long firstAt) {
      this.firstAt = firstAt;
      this.lastAt = firstAt;
    }

    private boolean isEmpty() {
      return pending.isEmpty() && overflow.isEmpty();
    }

    private Batch toBatch() {
      return new Batch(new ArrayList<>(pending.values()), overflow.size());
    }
  }
}
