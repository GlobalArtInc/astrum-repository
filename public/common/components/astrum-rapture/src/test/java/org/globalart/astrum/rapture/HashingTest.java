
package org.globalart.astrum.rapture;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import com.google.gson.Gson;
import org.junit.Test;

import com.google.common.hash.Hashing;
import com.google.gson.GsonBuilder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Hashing trials.
 */
public class HashingTest
{
  private static final Logger log = LoggerFactory.getLogger(HashingTest.class);

  @Test
  public void testGsonHash() {
    Gson gson = new GsonBuilder().create();
    List<String> data = Arrays.asList("a", "b", "c");
    String hash = Hashing.sha1().hashString(gson.toJson(data), StandardCharsets.UTF_8).toString();
    log.info(hash);
    assertEquals("e13460afb1e68af030bb9bee8344c274494661fa", hash);
  }

  @Test
  public void testObjectHash() {
    List<String> data = new ArrayList<>(Arrays.asList("a", "b", "c"));
    String initial = String.valueOf(data.hashCode());
    log.info(initial);
    data.add("d");
    String after = String.valueOf(data.hashCode());
    log.info(after);
    assertNotEquals(initial, after);
  }
}
