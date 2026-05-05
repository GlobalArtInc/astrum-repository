package org.globalart.astrum.repository.cargo.internal;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertThrows;

public class CargoPublishRequestTest
{
  static byte[] body(final String json, final byte[] crate) {
    byte[] metadata = json.getBytes(UTF_8);
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    out.writeBytes(ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(metadata.length).array());
    out.writeBytes(metadata);
    out.writeBytes(ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(crate.length).array());
    out.writeBytes(crate);
    return out.toByteArray();
  }

  @Test
  public void readsMetadataAndCrate() throws IOException {
    byte[] crate = "crate-bytes".getBytes(UTF_8);
    CargoPublishRequest request = CargoPublishRequest.read(
        new ByteArrayInputStream(body("{\"name\":\"Hello\",\"vers\":\"0.1.0\",\"deps\":[]}", crate)));
    assertThat(request.getCoordinates().getName(), is(equalTo("Hello")));
    assertThat(request.getCoordinates().getVersion(), is(equalTo("0.1.0")));
    assertThat(request.getCrateLength(), is(equalTo((long) crate.length)));
    assertThat(new String(request.getCrate().readAllBytes(), UTF_8), is(equalTo("crate-bytes")));
  }

  @Test
  public void rejectsInvalidRequests() {
    assertThrows(CargoException.class, () -> CargoPublishRequest.read(
        new ByteArrayInputStream(body("{\"name\":\"../x\",\"vers\":\"0.1.0\"}", new byte[]{1}))));
    assertThrows(CargoException.class, () -> CargoPublishRequest.read(
        new ByteArrayInputStream(body("{\"name\":\"x\",\"vers\":\"one\"}", new byte[]{1}))));
    assertThrows(CargoException.class, () -> CargoPublishRequest.read(
        new ByteArrayInputStream(body("[]", new byte[]{1}))));
    assertThrows(CargoException.class, () -> CargoPublishRequest.read(
        new ByteArrayInputStream(body("{\"name\":\"x\",\"vers\":\"0.1.0\"}", new byte[0]))));
    assertThrows(CargoException.class, () -> CargoPublishRequest.read(new ByteArrayInputStream(new byte[]{1, 0})));
  }
}
