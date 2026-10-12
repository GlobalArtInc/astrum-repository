package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.ByteArrayOutputStream;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.rubygems.internal.gem.GemDependency;
import org.globalart.astrum.repository.rubygems.internal.marshal.RubyMarshalWriter;

public final class DependencyApi
{
  public static final int MAX_GEMS = 200;

  private DependencyApi() {
  }

  public static byte[] marshal(final Map<String, List<InfoEntry>> entriesByName) {
    int count = entriesByName.values().stream().mapToInt(List::size).sum();
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    RubyMarshalWriter writer = new RubyMarshalWriter(out);
    writer.array(count);
    entriesByName.forEach((name, entries) -> {
      for (InfoEntry entry : entries) {
        writer.hash(4);
        writer.symbol("name").string(name);
        writer.symbol("number").string(entry.version());
        writer.symbol("platform").string(entry.platform());
        writer.symbol("dependencies").array(entry.dependencies().size());
        for (GemDependency dependency : entry.dependencies()) {
          writer.array(2).string(dependency.name()).string(String.join(", ", dependency.requirements()));
        }
      }
    });
    return out.toByteArray();
  }
}
