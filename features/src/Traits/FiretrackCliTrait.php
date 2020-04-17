<?php

declare(strict_types = 1);

namespace Firetrack\Tests\Traits;

use Firetrack\Tests\Exception\ExpectationException;

/**
 * Reusable code for interacting with the Firetrack CLI.
 */
trait FiretrackCliTrait
{

    /**
     * Executes a command using the Firetrack binary.
     *
     * @param string $command
     *   The command to execute.
     *
     * @return string[]
     *   The command output.
     */
    protected function executeCommand(string $command): array {
        $output = [];
        $exit = 0;

        $binary = $this->locateExecutable();
        $command = escapeshellcmd("$binary $command");
        exec($command, $output, $exit);

        if ($exit !== 0) {
            throw new ExpectationException(sprintf("Command '%s' returned error code %u.", $command, $exit));
        }

        return $output;
    }

    /**
     * Locates the Firetrack executable.
     *
     * @return string
     *   The absolute path to the Firetrack binary.
     */
    protected function locateExecutable(): string {
        foreach ($this->getBinaryLocations() as $location) {
            $path = getcwd() . DIRECTORY_SEPARATOR . $location;
            if (is_executable($path)) {
                return realpath($path);
            }
        }

        throw new ExpectationException(sprintf('Firetrack executable could not be located.'));
    }

    /**
     * Returns the possible locations of the firetrack binary, relative to the current path, in order of preference.
     *
     * @return string[]
     */
    protected function getBinaryLocations(): array {
        return [
            'target/release/cli',
            'target/debug/cli',
        ];
    }

}
