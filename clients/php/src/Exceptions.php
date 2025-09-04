<?php

declare(strict_types=1);

namespace MerkleKV;

use Exception;

/**
 * Base exception for all MerkleKV client errors.
 */
class MerkleKvException extends Exception
{
}

/**
 * Exception thrown when connection-related errors occur.
 */
class ConnectionException extends MerkleKvException
{
}

/**
 * Exception thrown when an operation times out.
 */
class TimeoutException extends MerkleKvException
{
}

/**
 * Exception thrown when a protocol error occurs (ERROR response from server).
 */
class ProtocolException extends MerkleKvException
{
}
