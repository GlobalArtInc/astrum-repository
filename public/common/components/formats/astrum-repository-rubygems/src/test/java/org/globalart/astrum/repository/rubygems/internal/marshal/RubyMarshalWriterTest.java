package org.globalart.astrum.repository.rubygems.internal.marshal;

import java.io.ByteArrayOutputStream;
import java.util.HexFormat;
import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RubyMarshalWriterTest
{
  private final ByteArrayOutputStream out = new ByteArrayOutputStream();

  private final RubyMarshalWriter writer = new RubyMarshalWriter(out);

  private String hex() {
    return HexFormat.of().formatHex(out.toByteArray());
  }

  @Test
  void writesSpecTuplesLikeRuby() {
    writer.array(1).array(3).string("rake").gemVersion("13.0.1").string("ruby");

    assertThat(hex(), is("04085b065b0849220972616b65063a064554553a1147656d3a3a56657273696f6e5b0649220b31332e302e31"
        + "063b005449220972756279063b0054"));
  }

  @Test
  void writesIntegersLikeRuby() {
    long[] values = {0, 1, 122, 123, -1, -123, -124, 255, 256, -256, 65536, (1 << 30) - 1};
    writer.array(values.length);
    for (long value : values) {
      writer.integer(value);
    }

    assertThat(hex(), is("04085b1169006906697f69017b69fa698069ff846901ff6902000169ff0069030000016904ffffff3f"));
  }

  @Test
  void writesUtf8Strings() {
    writer.string("héllo");

    assertThat(hex(), is("040849220b68c3a96c6c6f063a064554"));
  }

  @Test
  void writesRequirementsLikeRuby() {
    writer.gemRequirement(List.of(">= 10", "< 14"));

    assertThat(hex(), is("0408553a1547656d3a3a526571756972656d656e745b065b075b074922073e3d063a064554553a1147656d3a3a"
        + "56657273696f6e5b064922073130063b06545b074922063c063b0654553b075b064922073134063b0654"));
  }

  @Test
  void writesSymbolHashesLikeRuby() {
    writer.array(1).hash(4);
    writer.symbol("name").string("a");
    writer.symbol("number").string("1.0");
    writer.symbol("platform").string("ruby");
    writer.symbol("dependencies").array(1).array(2).string("b").string(">= 1, < 2");

    assertThat(hex(), is("04085b067b093a096e616d6549220661063a0645543a0b6e756d626572492208312e30063b06543a0d706c6174666f"
        + "726d49220972756279063b06543a11646570656e64656e636965735b065b0749220662063b065449220e3e3d20312c203c2032063b0654"));
  }

  @Test
  void rejectsIntegersOutsideTheFixnumRange() {
    assertThrows(IllegalArgumentException.class, () -> writer.integer(1L << 31));
  }
}
