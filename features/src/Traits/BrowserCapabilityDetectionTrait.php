<?php

declare(strict_types = 1);

namespace Firetrack\Tests\Traits;

use Behat\Mink\Exception\DriverException;
use Behat\Mink\Exception\UnsupportedDriverActionException;

/**
 * Helper methods for detecting browser capabilities.
 *
 * Taken from https://github.com/ec-europa/joinup-dev/blob/develop/tests/src/Traits/BrowserCapabilityDetectionTrait.php
 */
trait BrowserCapabilityDetectionTrait
{

    /**
     * Checks whether the browser supports JavaScript.
     *
     * @return bool
     *   Returns TRUE when the browser environment supports executing JavaScript
     *   code, for example because the test is running in Selenium or PhantomJS.
     */
    protected function browserSupportsJavaScript(): bool
    {
        $driver = $this->getSession()->getDriver();
        try {
            $driver->executeScript('return;');
            return true;
        } catch (UnsupportedDriverActionException|DriverException $e) {
            return false;
        }
    }

    /**
     * Checks that we are running on a JavaScript-enabled browser.
     *
     * @throws \LogicException
     *   Thrown when not running on a JS-enabled browser.
     */
    protected function assertJavaScriptEnabledBrowser(): void
    {
        if (!$this->browserSupportsJavaScript()) {
            // Show a helpful error message.
            throw new \LogicException('This test needs to run on a real browser using Selenium or similar. Please add the "@javascript" tag to the scenario.');
        }
    }

}
