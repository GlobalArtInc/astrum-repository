package org.globalart.astrum.notifications.internal;

import java.util.Map;

import org.globalart.astrum.notifications.internal.NotificationBuffer.Batch;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.aMapWithSize;
import static org.hamcrest.Matchers.anEmptyMap;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;

public class NotificationBufferTest
{
  private final NotificationBuffer buffer = new NotificationBuffer(10_000, 60_000, 3);

  private static ComponentNotice notice(final String name, final NoticeAction action) {
    return new ComponentNotice("maven-releases", "maven2", "org.example", name, "1.0", action, "roman", null, null,
        null, "hosted", null);
  }

  @Test
  public void waitsUntilTheChannelIsQuiet() {
    buffer.add("c1", notice("demo", NoticeAction.PUBLISHED), 0);
    buffer.add("c1", notice("demo", NoticeAction.UPDATED), 4_000);
    buffer.add("c1", notice("demo", NoticeAction.UPDATED), 8_000);

    assertThat(buffer.drainReady(12_000), is(anEmptyMap()));

    Map<String, Batch> ready = buffer.drainReady(18_000);
    assertThat(ready, aMapWithSize(1));
    assertThat(ready.get("c1").notices(), contains(notice("demo", NoticeAction.PUBLISHED)));
    assertThat(buffer.isEmpty(), is(true));
  }

  @Test
  public void aBusyChannelIsFlushedAfterTheMaximumDelay() {
    for (long now = 0; now <= 60_000; now += 5_000) {
      buffer.add("c1", notice("demo", NoticeAction.UPDATED), now);
    }

    assertThat(buffer.drainReady(59_999), is(anEmptyMap()));
    assertThat(buffer.drainReady(60_000).get("c1").notices(), contains(notice("demo", NoticeAction.UPDATED)));
  }

  @Test
  public void mergesTheChangesOfOneComponent() {
    buffer.add("c1", notice("published-then-updated", NoticeAction.PUBLISHED), 0);
    buffer.add("c1", notice("published-then-updated", NoticeAction.UPDATED), 0);
    buffer.add("c1", notice("deleted-then-published", NoticeAction.DELETED), 0);
    buffer.add("c1", notice("deleted-then-published", NoticeAction.PUBLISHED), 0);
    buffer.add("c1", notice("updated-then-deleted", NoticeAction.UPDATED), 0);
    buffer.add("c1", notice("updated-then-deleted", NoticeAction.DELETED), 0);

    assertThat(buffer.drainReady(10_000).get("c1").notices(), contains(
        notice("published-then-updated", NoticeAction.PUBLISHED),
        notice("deleted-then-published", NoticeAction.UPDATED),
        notice("updated-then-deleted", NoticeAction.DELETED)));
  }

  @Test
  public void aComponentPublishedAndDeletedInOneWindowIsNotTold() {
    buffer.add("c1", notice("demo", NoticeAction.PUBLISHED), 0);
    buffer.add("c1", notice("demo", NoticeAction.DELETED), 1_000);

    assertThat(buffer.drainReady(20_000), is(anEmptyMap()));
  }

  @Test
  public void keepsTheLatestKnownDetails() {
    buffer.add("c1", notice("demo", NoticeAction.PUBLISHED), 0);
    buffer.add("c1", new ComponentNotice("maven-releases", "maven2", "org.example", "demo", "1.0",
        NoticeAction.UPDATED, null, "org/example/demo/1.0", null, "http://host", "hosted", null), 0);

    ComponentNotice merged = buffer.drainReady(10_000).get("c1").notices().get(0);
    assertThat(merged.action(), is(NoticeAction.PUBLISHED));
    assertThat(merged.initiator(), is("roman"));
    assertThat(merged.browsePath(), is("org/example/demo/1.0"));
    assertThat(merged.baseUrl(), is("http://host"));
  }

  @Test
  public void channelsHaveSeparateWindows() {
    buffer.add("c1", notice("a", NoticeAction.PUBLISHED), 0);
    buffer.add("c2", notice("b", NoticeAction.PUBLISHED), 5_000);

    Map<String, Batch> first = buffer.drainReady(10_000);
    assertThat(first.keySet(), contains("c1"));
    Map<String, Batch> second = buffer.drainReady(15_000);
    assertThat(second.keySet(), contains("c2"));
  }

  @Test
  public void countsComponentsBeyondTheCapacity() {
    for (int i = 0; i < 5; i++) {
      buffer.add("c1", notice("lib-" + i, NoticeAction.PUBLISHED), 0);
      buffer.add("c1", notice("lib-" + i, NoticeAction.UPDATED), 0);
    }

    Batch batch = buffer.drainReady(10_000).get("c1");
    assertThat(batch.notices(), hasSize(3));
    assertThat(batch.overflow(), is(2));
  }

  @Test
  public void drainAllReturnsEverythingPending() {
    buffer.add("c1", notice("a", NoticeAction.PUBLISHED), 0);

    assertThat(buffer.drainAll().get("c1").notices(), hasSize(1));
    assertThat(buffer.isEmpty(), is(true));
  }
}
