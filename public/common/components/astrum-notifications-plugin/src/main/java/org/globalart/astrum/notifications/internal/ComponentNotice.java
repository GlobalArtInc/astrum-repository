package org.globalart.astrum.notifications.internal;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.nullToEmpty;

/**
 * A change of one component, captured when the repository reported it and rendered later into a message.
 *
 * @param repository the repository name
 * @param format the repository format, e.g. {@code maven2}
 * @param namespace the component namespace, empty when the format has none
 * @param name the component name
 * @param version the component version, empty when the format has none
 * @param action what happened to the component
 * @param initiator the user who made the change, if known
 * @param browsePath the component's node in the repository browse tree, encoded the way the browse tree encodes node
 *          ids, if known
 * @param registry the Docker registry address that serves the repository, if known
 * @param baseUrl the base URL of the user interface, if known
 * @param repositoryType the repository type: {@code hosted}, {@code proxy} or {@code group}
 * @param componentId the component's id in the REST API, if known
 */
public record ComponentNotice(
    String repository,
    String format,
    String namespace,
    String name,
    String version,
    NoticeAction action,
    @Nullable String initiator,
    @Nullable String browsePath,
    @Nullable String registry,
    @Nullable String baseUrl,
    String repositoryType,
    @Nullable String componentId)
{
  public ComponentNotice {
    checkNotNull(repository);
    checkNotNull(name);
    checkNotNull(action);
    format = nullToEmpty(format);
    namespace = nullToEmpty(namespace);
    version = nullToEmpty(version);
    repositoryType = nullToEmpty(repositoryType);
  }

  /**
   * Identifies the component across delete and re-create, unlike its database id.
   */
  public String key() {
    return repository + '\n' + namespace + '\n' + name + '\n' + version;
  }

  public ComponentNotice withAction(final NoticeAction newAction) {
    return new ComponentNotice(repository, format, namespace, name, version, newAction, initiator, browsePath, registry,
        baseUrl, repositoryType, componentId);
  }

  /**
   * Combines this change with a later change of the same component within one delivery window: a component that is
   * published and then updated is still published, one that is deleted and published again is updated, and one that
   * is published and deleted again leaves nothing to tell.
   *
   * @return the combined change, or {@code null} when nothing is left to tell
   */
  @Nullable
  public ComponentNotice merge(final ComponentNotice later) {
    NoticeAction merged = switch (action) {
      case PUBLISHED -> later.action == NoticeAction.DELETED ? null : NoticeAction.PUBLISHED;
      case UPDATED -> later.action;
      case DELETED -> later.action == NoticeAction.DELETED ? NoticeAction.DELETED : NoticeAction.UPDATED;
    };
    if (merged == null) {
      return null;
    }
    return new ComponentNotice(
        repository,
        later.format.isEmpty() ? format : later.format,
        namespace,
        name,
        version,
        merged,
        later.initiator != null ? later.initiator : initiator,
        later.browsePath != null ? later.browsePath : browsePath,
        later.registry != null ? later.registry : registry,
        later.baseUrl != null ? later.baseUrl : baseUrl,
        later.repositoryType.isEmpty() ? repositoryType : later.repositoryType,
        later.componentId != null ? later.componentId : componentId);
  }
}
