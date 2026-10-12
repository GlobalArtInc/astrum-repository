package org.globalart.astrum.repository.swift.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftRecipeSupport;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(SwiftGroupRecipe.NAME)
public class SwiftGroupRecipe
    extends SwiftRecipeSupport
{
  public static final String NAME = "swift-group";

  private final Provider<GroupFacet> groupFacet;

  private final SwiftGroupHandlers.Releases releasesHandler;

  private final SwiftGroupHandlers.Identifiers identifiersHandler;

  private final SwiftGroupHandlers.FirstMember firstMemberHandler;

  private final SwiftGroupWriteHandler writeHandler;

  @Autowired
  public SwiftGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(SwiftFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      final SwiftGroupHandlers.Releases releasesHandler,
      final SwiftGroupHandlers.Identifiers identifiersHandler,
      final SwiftGroupHandlers.FirstMember firstMemberHandler,
      final SwiftGroupWriteHandler writeHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.releasesHandler = checkNotNull(releasesHandler);
    this.identifiersHandler = checkNotNull(identifiersHandler);
    this.firstMemberHandler = checkNotNull(firstMemberHandler);
    this.writeHandler = checkNotNull(writeHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(groupFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addCommonRoutes(builder);
    builder.route(apiRoute(SwiftPaths.identifiers(), identifiersHandler));
    builder.route(apiRoute(SwiftPaths.publish(), writeHandler));
    builder.route(contentRoute(SwiftPaths.archive(), firstMemberHandler));
    builder.route(contentRoute(SwiftPaths.manifest(), firstMemberHandler));
    builder.route(contentRoute(SwiftPaths.releases(), releasesHandler));
    builder.route(contentRoute(SwiftPaths.release(), firstMemberHandler));
    addDefaultRoutes(builder);
    facet.configure(builder.create());
    return facet;
  }
}
