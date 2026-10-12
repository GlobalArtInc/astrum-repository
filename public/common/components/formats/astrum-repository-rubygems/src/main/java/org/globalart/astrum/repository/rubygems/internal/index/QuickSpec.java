package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.List;
import java.util.Map;
import java.util.zip.Deflater;
import java.util.zip.DeflaterOutputStream;

import org.globalart.astrum.repository.rubygems.internal.gem.GemDependency;
import org.globalart.astrum.repository.rubygems.internal.gem.GemPlatform;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.rubygems.internal.marshal.RubyMarshalWriter;

public final class QuickSpec
{
  private static final String DEFAULT_RUBYGEMS_VERSION = "3.0.0";

  private QuickSpec() {
  }

  public static byte[] deflated(final GemSpec spec) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (DeflaterOutputStream deflater = new DeflaterOutputStream(out, new Deflater(Deflater.BEST_COMPRESSION))) {
      deflater.write(marshal(spec));
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    return out.toByteArray();
  }

  public static byte[] marshal(final GemSpec spec) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    new RubyMarshalWriter(out).userDefined("Gem::Specification", fields(spec));
    return out.toByteArray();
  }

  static byte[] fields(final GemSpec spec) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    RubyMarshalWriter writer = new RubyMarshalWriter(out);
    writer.array(19);
    writer.string(spec.rubygemsVersion() != null ? spec.rubygemsVersion() : DEFAULT_RUBYGEMS_VERSION);
    writer.integer(spec.specificationVersion());
    writer.string(spec.name());
    writer.gemVersion(spec.version());
    if (spec.date() != null) {
      writer.string(spec.date());
    }
    else {
      writer.nil();
    }
    writer.string(spec.summary() != null ? spec.summary() : "");
    writer.gemRequirement(spec.requiredRubyVersion());
    writer.gemRequirement(spec.requiredRubygemsVersion());
    writer.string(spec.platform());
    List<GemDependency> dependencies = spec.dependencies();
    writer.array(dependencies.size());
    for (GemDependency dependency : dependencies) {
      writer.object("Gem::Dependency", 4);
      writer.symbol("@name").string(dependency.name());
      writer.symbol("@requirement").gemRequirement(dependency.requirements());
      writer.symbol("@type").symbol(dependency.isRuntime() ? GemDependency.RUNTIME : GemDependency.DEVELOPMENT);
      writer.symbol("@prerelease").bool(false);
    }
    writer.string("");
    writer.strings(spec.email());
    writer.strings(spec.authors());
    writer.string(spec.description() != null ? spec.description() : "");
    writer.string(spec.homepage() != null ? spec.homepage() : "");
    writer.bool(true);
    if (GemPlatform.isRuby(spec.platform())) {
      writer.string(GemPlatform.RUBY);
    }
    else {
      GemPlatform platform = GemPlatform.parse(spec.platform());
      writer.object("Gem::Platform", 3);
      writer.symbol("@cpu").string(platform.cpu());
      writer.symbol("@os").string(platform.os());
      writer.symbol("@version").string(platform.version());
    }
    writer.strings(spec.licenses());
    Map<String, String> metadata = spec.metadata();
    writer.hash(metadata.size());
    metadata.forEach((key, value) -> writer.string(key).string(value));
    return out.toByteArray();
  }
}
