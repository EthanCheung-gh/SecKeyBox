import { beforeEach, describe, expect, it } from 'vitest';
import { useUIStore } from '@/stores/ui';

describe('useUIStore view selection', () => {
  beforeEach(() => {
    useUIStore.setState({
      selectedGroupId: null,
      selectedTagId: null,
      showFavoritesOnly: false,
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
});
