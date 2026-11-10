import { beforeEach, describe, expect, it } from 'vitest';
import { useUIStore } from '@/stores/ui';

describe('useUIStore view selection', () => {
  beforeEach(() => {
    useUIStore.setState({
      selectedGroupId: null,
      selectedTagId: null,
      showFavoritesOnly: false,
      isSecurityPanelOpen: false,
      isTrashOpen: false,
    });
  });

  it('clears tag and favorites when a group is selected', () => {
    useUIStore.setState({ selectedTagId: 'tag-1', showFavoritesOnly: true });

    useUIStore.getState().setSelectedGroup('group-1');

    expect(useUIStore.getState()).toMatchObject({
      selectedGroupId: 'group-1',
      selectedTagId: null,
      showFavoritesOnly: false,
    });
  });

  it('clears group and favorites when a tag is selected', () => {
    useUIStore.setState({ selectedGroupId: 'group-1', showFavoritesOnly: true });

    useUIStore.getState().setSelectedTag('tag-1');

    expect(useUIStore.getState()).toMatchObject({
      selectedGroupId: null,
      selectedTagId: 'tag-1',
      showFavoritesOnly: false,
    });
  });

  it('clears group and tag when favorites-only is enabled', () => {
    useUIStore.setState({ selectedGroupId: 'group-1', selectedTagId: 'tag-1' });

    useUIStore.getState().setShowFavoritesOnly(true);

    expect(useUIStore.getState()).toMatchObject({
      selectedGroupId: null,
      selectedTagId: null,
      showFavoritesOnly: true,
    });
  });

  it('closes the security panel and keeps the group when the trash opens', () => {
    useUIStore.setState({
      selectedGroupId: 'group-1',
      selectedTagId: null,
      showFavoritesOnly: false,
      isSecurityPanelOpen: true,
    });

    useUIStore.getState().openTrash();

    expect(useUIStore.getState()).toMatchObject({
      isTrashOpen: true,
      isSecurityPanelOpen: false,
      selectedGroupId: 'group-1',
    });
  });

  it('closes the trash and the security panel when a group is selected', () => {
    useUIStore.setState({ isTrashOpen: true, isSecurityPanelOpen: true });

    useUIStore.getState().setSelectedGroup('group-1');

    expect(useUIStore.getState()).toMatchObject({
      isTrashOpen: false,
      isSecurityPanelOpen: false,
      selectedGroupId: 'group-1',
    });
  });

  it('closes the trash when the security panel opens', () => {
    useUIStore.setState({ isTrashOpen: true });

    useUIStore.getState().openSecurityPanel();

    expect(useUIStore.getState()).toMatchObject({
      isSecurityPanelOpen: true,
      isTrashOpen: false,
    });
  });

  it('clears the trash flag on close without touching the selection', () => {
    useUIStore.setState({ isTrashOpen: true, selectedGroupId: 'group-1' });

    useUIStore.getState().closeTrash();

    expect(useUIStore.getState()).toMatchObject({
      isTrashOpen: false,
      selectedGroupId: 'group-1',
    });
  });
});
