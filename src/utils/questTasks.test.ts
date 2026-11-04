import { describe, expect, it } from 'vitest'
import type { Quest } from '@/api/tauri'
import en from '@/locales/en.json'
import zh from '@/locales/zh.json'
import {
  firstProgressValue,
  firstStartableTask,
  getQuestActionKind,
  getQuestKind,
  getQuestTasks,
  isPlayActivityTask,
  playActivityProgressPercentage,
} from './questTasks'

function playActivityQuest(): Quest {
  return {
    id: 'cloud-activity',
    config: {
      messages: { quest_name: 'Cloud Activity' },
      task_config_v2: {
        tasks: {
          PLAY_ACTIVITY: { type: 'PLAY_ACTIVITY', target: 900 },
        },
      },
    },
    user_status: {
      enrolled_at: '2026-08-04T00:00:00.000Z',
      progress: {
        PLAY_ACTIVITY: { value: 48 },
      },
    },
  }
}

function desktopQuest(id: string, type: string): Quest {
  return {
    id,
    config: {
      messages: { quest_name: id },
      task_config_v2: {
        tasks: {
          [type]: { type, target: 900 },
        },
      },
    },
    user_status: null,
  }
}

describe('PLAY_ACTIVITY task helpers', () => {
  it('keeps cloud games in the Activity kind with a distinct task predicate', () => {
    const quest = playActivityQuest()
    const task = firstStartableTask(quest)

    expect(getQuestKind(quest)).toBe('activity')
    expect(task?.type).toBe('PLAY_ACTIVITY')
    expect(task && isPlayActivityTask(task)).toBe(true)
    expect(getQuestTasks(quest)[0].label).toBe('Activity - Cloud Game')
  })

  it('restores PLAY_ACTIVITY progress as elapsed seconds', () => {
    const quest = playActivityQuest()

    expect(firstProgressValue(quest, 'PLAY_ACTIVITY')).toBe(48)
  })

  it('does not report completion before Discord sets completed_at', () => {
    expect(playActivityProgressPercentage(450, 900)).toBe(50)
    expect(playActivityProgressPercentage(900, 900)).toBe(99)
    expect(playActivityProgressPercentage(-1, 900)).toBe(0)
    expect(playActivityProgressPercentage(900, 0)).toBe(0)
    expect(playActivityProgressPercentage(900, 900, true)).toBe(100)
  })

  it('provides localized cloud-game badge labels', () => {
    expect(en.filter.activity_cloud_game).toBe('Activity - Cloud Game')
    expect(zh.filter.activity_cloud_game).toBe('活动 - 云游戏')
  })
})

describe('quest action routing', () => {
  it('routes desktop play and stream quests to different runners', () => {
    const play = desktopQuest('play', 'PLAY_ON_DESKTOP')
    const stream = desktopQuest('stream', 'STREAM_ON_DESKTOP')

    // Both stay in the UI "game" kind, so execution cannot rely on it.
    expect(getQuestKind(play)).toBe('stream')
    expect(getQuestKind(stream)).toBe('stream')

    expect(getQuestActionKind(play)).toBe('play')
    expect(getQuestActionKind(stream)).toBe('stream')
  })

  it('separates cloud activities from checkpoint activities', () => {
    expect(getQuestActionKind(playActivityQuest())).toBe('play_activity')
    expect(getQuestActionKind(desktopQuest('checkpoint', 'ACHIEVEMENT_IN_ACTIVITY'))).toBe('activity')
  })

  it('returns null when no task can be started', () => {
    expect(getQuestActionKind({ id: 'empty', config: {} } as Quest)).toBeNull()
  })
})
