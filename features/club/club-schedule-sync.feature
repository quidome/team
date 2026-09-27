Feature: Club schedule sync
  The coordinator keeps the club's games in step with the federation schedule feed.
  Clubs, teams, games, and locations keep their own identity in the app and remember the source IDs they were matched to.

  Scenario: Importing the club's games for a season
    Given the club Blue Drakes is configured with source club ID 37
    And the feed lists 71 games for Blue Drakes in season 2026-2027
    When the coordinator syncs the club schedule
    Then 71 games are stored, each with its source game ID
    And each game has a home team, an away team, a date, a start time, and a location

  Scenario: Recognising the club's own teams
    Given the feed lists games for the Blue Drakes teams M16 1, M16 2, M18 1, MSE 1, X12 1, and X14 1
    When the coordinator syncs the club schedule
    Then each of those teams is stored as a team of the club
    And opponent teams are stored as teams of their own clubs

  Scenario: Keeping a team when its source team ID changes
    Given U16-1 is linked to source team ID 184 for the first half of the season
    And the feed lists a second-half game for Blue Drakes M16 1 with source team ID 912
    When the coordinator links source team ID 912 to U16-1
    Then first-half and second-half games both belong to U16-1
    And the players and duty history of U16-1 are unchanged

  Scenario: Updating a rescheduled game
    Given a synced game with source game ID 1073150 is scheduled on 2026-10-03 at 16:00
    And the feed now lists that game on 2026-10-10 at 15:00
    When the coordinator syncs the club schedule
    Then the original occurrence remains stored as cancelled
    And a new scheduled occurrence is stored for the same game

  Scenario: Syncing twice does not duplicate games
    Given the club schedule was synced
    When the coordinator syncs the club schedule again without feed changes
    Then no games are added or changed

  Scenario: Flagging a game that disappeared from the feed
    Given a synced game is no longer listed in the feed
    When the coordinator syncs the club schedule
    Then the game is kept and marked as missing from the source
    And the coordinator decides whether to cancel it

  Scenario: Handling a game without a known start time
    Given the feed lists a game at 00:00
    When the coordinator syncs the club schedule
    Then the game is stored without a start time
    And no departure time is suggested for it

  Scenario: Ignoring placeholder results for unplayed games
    Given the feed lists an unplayed game with the result 0 - 0
    When the coordinator syncs the club schedule
    Then the game has no result
