#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class Action : uint8_t {
  BasicSynthesis,
  BasicTouch,
  MasterMend,
  Observe,
  TricksOfTheTrade,
  WasteNot,
  Veneration,
  StandardTouch,
  GreatStrides,
  Innovation,
  WasteNot2,
  ByregotsBlessing,
  PreciseTouch,
  MuscleMemory,
  CarefulSynthesis,
  Manipulation,
  PrudentTouch,
  AdvancedTouch,
  Reflect,
  PreparatoryTouch,
  Groundwork,
  DelicateSynthesis,
  IntensiveSynthesis,
  TrainedEye,
  HeartAndSoul,
  PrudentSynthesis,
  TrainedFinesse,
  RefinedTouch,
  QuickInnovation,
  ImmaculateMend,
  TrainedPerfection,
};

enum class LevelFilter : uint8_t {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
};

struct SolveArgs {
  void (*on_start)(bool*);
  void (*on_finish)(const Action*, size_t);
  void (*on_suggest_solution)(const Action*, size_t);
  void (*on_progress)(size_t);
  void (*on_log)(const uint8_t*, size_t);
  LevelFilter log_level;
  uint16_t thread_count;
  uint64_t action_mask;
  uint16_t progress;
  uint16_t quality;
  uint16_t base_progress;
  uint16_t base_quality;
  uint16_t cp;
  uint16_t durability;
  uint8_t job_level;
  bool adversarial;
  bool backload_progress;
};

extern "C" {

void solve(const SolveArgs *args);

}  // extern "C"
