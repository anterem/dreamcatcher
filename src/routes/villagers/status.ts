import type { Villager } from '../../lib/bindings';
import { last5amUtc } from '../../lib/time';

export type VillagerStatus = Villager & { giftsAreCurrent: boolean; giftable: boolean };

export function villagerStatus(
  villager: Villager,
  modifiedSecs: number,
  nowSecs: number,
  tzOffset: number
): VillagerStatus {
  const giftsAreCurrent = modifiedSecs >= last5amUtc(nowSecs, tzOffset);
  return {
    ...villager,
    giftsAreCurrent,
    giftable: giftsAreCurrent && villager.gifts.some((g) => !g.gifted)
  };
}
