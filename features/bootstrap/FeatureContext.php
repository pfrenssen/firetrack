<?php

declare(strict_types = 1);

use Behat\Behat\Context\Context;

/**
 * Step definitions specific to the application.
 */
class FeatureContext implements Context
{

    /**
     * Pauses the scenario until the user presses a key.
     *
     * Useful when debugging a scenario.
     *
     * @Then I break
     *
     * @see https://github.com/jhedstrom/drupalextension/blob/master/src/Drupal/DrupalExtension/Context/DrupalContext.php
     */
    public function iPutABreakpoint()
    {
        fwrite(STDOUT, "\033[s \033[93m[Breakpoint] Press \033[1;93m[RETURN]\033[0;93m to continue, or 'q' to quit...\033[0m");
        do {
            $line = trim(fgets(STDIN, 1024));
            $char_code = ord($line);
            switch ($char_code) {
                case 0: //CR
                    break 2;
                case 113: //q
                case 81: //Q
                    throw new \Exception("Exiting test intentionally.");
                default:
                    fwrite(STDOUT, sprintf("\nInvalid entry '%s'.  Please enter 'y', 'q', or the enter key.\n", $line));
                    break;
            }
        } while (true);
        fwrite(STDOUT, "\033[u");
    }
}
